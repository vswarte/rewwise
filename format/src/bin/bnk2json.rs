use std::collections;
use std::env;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path;

use deku::bitvec::BitVec;
use deku::DekuWrite;
use wwise_format::DATASection;
use wwise_format::DIDXDescriptor;
use wwise_format::DIDXSection;
use wwise_format::ObjectId;
use wwise_format::Section;
use wwise_format::SectionBody;
use wwise_format::Soundbank;

fn main() {
    let args: Vec<String> = env::args().collect();

    for path in args[1..].iter() {
        let path = path::PathBuf::from(path);
        let md = fs::metadata(&path).unwrap();

        if md.is_file() {
            handle_soundbank(path);
        } else if md.is_dir() {
            handle_dir(path);
        } else {
            panic!("Was unable to handle path {path:?}");
        }
    }
}

pub type FNVDictionary = collections::HashMap<u32, String>;

pub fn parse_dictionary(input: &str) -> FNVDictionary {
    input
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| (ObjectId::String(l.to_string()).as_hash(), l.to_string()))
        .collect()
}

fn handle_soundbank(path: path::PathBuf) {
    let dictionary_path = {
        let mut tmp = std::env::current_dir()
            .expect("Could not determine execution directory");
        tmp.push("dictionary.txt");
        tmp
    };

    let dictionary = if let Ok(dictionary) = std::fs::read_to_string(dictionary_path) {
        parse_dictionary(&dictionary)
    } else {
        parse_dictionary(include_str!("default_dictionary.txt"))
    };

    // Parse the soundbank
    let mut soundbank = {
        let mut handle = fs::File::open(&path).expect("Could not acquire read file handle");

        let mut file_buffer = vec![];
        handle
            .read_to_end(&mut file_buffer)
            .expect("Could not read input file");

        wwise_format::parse_soundbank(&file_buffer).expect("Could not parse bnk")
    };

    // Create output directory
    let output_dir = {
        let mut p = path.parent().unwrap().to_path_buf();

        p.push(path.file_stem().unwrap());

        fs::create_dir_all(&p).expect("Could not create output directory");

        p
    };

    {
        // Acquire DIDX and the DATA
        let didx = &soundbank.sections.iter().find_map(|s| match &s.body {
            wwise_format::SectionBody::DIDX(s) => Some(s),
            _ => None,
        });
        let data = &soundbank.sections.iter().find_map(|s| match &s.body {
            wwise_format::SectionBody::DATA(s) => Some(s),
            _ => None,
        });

        // If both are available start carving
        if didx.is_some() && data.is_some() {
            let didx = didx.unwrap();
            let data = data.unwrap();

            for descriptor in didx.descriptors.iter() {
                let mut file_path = output_dir.clone();
                file_path.push(format!("{}.wem", descriptor.id));

                let start = descriptor.offset as usize;
                let end = start + descriptor.size as usize;

                let bytes = &data.data[start..end];
                fs::write(file_path, bytes).expect("Could not write WEM to output directory");
            }
        }
    }

    // Remove DIDX and DATA from JSON output
    soundbank.sections.retain(|s| {
        !matches!(
            &s.body,
            wwise_format::SectionBody::DIDX(_) | wwise_format::SectionBody::DATA(_)
        )
    });

    if let Some(h) = soundbank
        .sections
        .iter_mut()
        .find_map(|s| match &mut s.body {
            SectionBody::HIRC(h) => Some(h),
            _ => None,
        })
    {
        for object in h.objects.iter_mut() {
            object.id = match dictionary.get(&object.id.as_hash()) {
                Some(s) => ObjectId::String(s.to_string()),
                None => object.id.clone(),
            };
        }
    }

    // Create the soundbank.json
    let mut json_path = output_dir.clone();
    json_path.push("soundbank.json");
    let handle = fs::File::create(&json_path).expect("could not acquire write file handle");

    serde_json::to_writer_pretty(handle, &soundbank).expect("could not write json to output file");
}

fn handle_dir(path: path::PathBuf) {
    // Parse soundbank JSON
    let mut soundbank = {
        let mut json_path = path.clone();
        json_path.push("soundbank.json");

        let handle = fs::File::open(&json_path).expect("Could not acquire read file handle");

        serde_json::from_reader::<_, Soundbank>(handle)
            .expect("Could not deserialize input into a soundbank")
    };

    // Get a directory listing
    let files = fs::read_dir(&path)
        .expect("Could not read unpacked soundbank director")
        .map(|f| f.unwrap().file_name().to_string_lossy().to_string())
        .collect::<Vec<String>>();

    // Find all the wems
    let mut wems = files
        .iter()
        .filter(|f| f.ends_with(".wem"))
        .collect::<Vec<_>>();

    // Sort the wems
    wems.sort_by(|a, b| {
        let a = a.replace(".wem", "").parse::<u32>().unwrap();
        let b = b.replace(".wem", "").parse::<u32>().unwrap();
        a.partial_cmp(&b).unwrap()
    });

    // Rebuild the DIDX and the DATA
    let mut descriptors = Vec::new();
    let mut data = Vec::new();
    let mut cursor = io::Cursor::new(&mut data);

    // Obtain the WEM alignment
    let wem_alignment = soundbank
        .sections
        .iter()
        .find_map(|f| match &f.body {
            SectionBody::BKHD(b) => Some(b),
            _ => None,
        })
        .expect("Soundbank needs a BKDH section")
        .wem_alignment;

    for (i, wem) in wems.iter().enumerate() {
        let id = wem
            .replace(".wem", "")
            .parse::<u32>()
            .expect("Could not parse WEM name to WEM ID");
        let offset = cursor.stream_position().unwrap() as u32;
        let wem_path = format!("{}/{}", path.to_string_lossy(), wem);

        // Write WEM bytes to DATA section buffer
        let file_bytes = fs::read(wem_path).expect("Could not read WEM file");

        cursor
            .write_all(&file_bytes)
            .expect("Could not write WEM to DATA buffer");

        let current_pos = cursor.stream_position().expect("Could not seek") as u32;
        let padded_position = (current_pos + wem_alignment - 1) & !(wem_alignment - 1);
        let bytes_to_pad = padded_position - current_pos;

        // Last WEM entry has no padding
        if i != wems.len() - 1 {
            for _ in 0..bytes_to_pad {
                _ = cursor.write(&[0]).expect("Could not write padding byte");
            }
        }

        let size = file_bytes.len() as u32;
        descriptors.push(DIDXDescriptor { id, offset, size });
    }

    if !descriptors.is_empty() {
        let didx = DIDXSection { descriptors };
        let data = DATASection { data };

        // Put the DIDX and the DATA after the BKHD but before anythign elsee
        // TODO: could use a deque instead of a vec?
        let mut sections = soundbank.sections;

        // Grab the BKHD
        sections.rotate_left(1);
        let bkhd = sections.pop().unwrap();

        // Append and rotate the DATA
        sections.push(Section {
            magic: [0x0; 4],
            size: 0,
            body: SectionBody::DATA(data),
        });
        sections.rotate_right(1);

        // Append and rotate the DIDX
        sections.push(Section {
            magic: [0x0; 4],
            size: 0,
            body: SectionBody::DIDX(didx),
        });
        sections.rotate_right(1);

        // Readd the NKHD
        sections.push(bkhd);
        sections.rotate_right(1);

        soundbank.sections = sections;
    }

    // Prepare soundbank JSON repr for its bin equivalent
    wwise_format::prepare_soundbank(&mut soundbank);

    // Write the soundbank to the bin buffer
    let mut soundbank_bytes = BitVec::default();
    soundbank
        .write(&mut soundbank_bytes, ())
        .expect("Could not encode soundbank to bytes");

    // Make output bnk file
    let mut bnk_path = path.clone();
    bnk_path.set_extension("created.bnk");

    let mut handle = fs::File::create(&bnk_path).expect("Could not acquire write file handle");

    handle
        .write_all(soundbank_bytes.as_raw_slice())
        .expect("Could not write to result file");
}

use std::fs;
use std::env;
use std::io::Write;
use std::path;
use std::io::Read;

use deku::DekuWrite;
use deku::bitvec::BitVec;
use wwise_format::Soundbank;

fn main() {
    let mut args: Vec<String> = env::args().collect();

    for path in args[1..].iter() {
        let path = path::PathBuf::from(path);

        let extension = path.extension().expect("Could not determine file type");
        if extension == "bnk" {
            handle_soundbank(path);
        } else if extension == "json" {
            handle_json(path);
        } else {
            panic!("Was unable to handle file type {}", extension.to_str().unwrap());
        }
    }
}

fn handle_soundbank(mut path: path::PathBuf) {
    let soundbank = {
        let mut handle = fs::File::open(&path)
            .expect("Could not acquire read file handle");

        let mut file_buffer = vec![];
        handle.read_to_end(&mut file_buffer)
            .expect("Could not read input file");

        wwise_format::parse_soundbank(&file_buffer)
            .expect("Could not parse bnk")
    };

    path.set_extension("bnk.json");

    let handle = fs::File::create(&path)
        .expect("Could not acquire write file handle");

    serde_json::to_writer_pretty(handle, &soundbank)
        .expect("Could not write JSON to output file");
}

fn handle_json(mut path: path::PathBuf) {
    let mut soundbank = {
        let handle = fs::File::open(&path)
            .expect("Could not acquire read file handle");

        serde_json::from_reader::<_, Soundbank>(handle)
            .expect("Could not deserialize input into a soundbank")
    };

    // Fill in fields that were skipped during serialization due to redundancy
    wwise_format::prepare_soundbank(&mut soundbank);

    path.set_extension("created.bnk");

    let mut soundbank_bytes = BitVec::default();
    soundbank.write(&mut soundbank_bytes, ())
        .expect("Could not encode soundbank to bytes");

    let mut handle = fs::File::create(&path)
        .expect("Could not acquire write file handle");

    handle.write_all(soundbank_bytes.as_raw_slice())
        .expect("Could not write to result file");
}

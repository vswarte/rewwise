use std::fs;
use std::io;
use std::env;
use deku::prelude::*;
use wwise_format::HIRCObject;

fn main() {
    let args: Vec<String> = env::args().collect();
    let paths = &args[1..];

    for path in paths {
        let mut handle = fs::File::open(path)
            .expect("Could not acquire file handle");

        println!("Checking {}", path);
        dissect_bnk(&mut handle)
            .expect("Could not read file for parsing");
    }
}

fn dissect_bnk(data: &mut (impl io::Read + io::Seek)) -> Result<(), io::Error> {
    let mut header_bytes = vec![0u8; 8];
    while data.read_exact(&mut header_bytes).is_ok() {
        let header = {
            let result = SectionHeader::from_bytes((header_bytes.as_slice(), 0))
                .expect("Could not parse header bytes");

            result.1
        };

        //println!("Discovered bnk section. magic = {}", String::from_utf8((&header.magic).to_vec()).unwrap());

        if &header.magic == b"HIRC" {
            // Read HIRC object count
            let count = {
                let mut count_buffer = vec![0u8; 4];
                data.read_exact(&mut count_buffer)?;
                let count_buffer: [u8; 4] = count_buffer.as_slice()[0..4].try_into().unwrap();
                u32::from_le_bytes(count_buffer)
            };

            for _ in 0..count {
                let (object_type, object_size, object_id)= {
                    let mut buffer = vec![0u8; 9];
                    data.read_exact(&mut buffer)?;

                    let type_buffer: [u8; 1] = buffer.as_slice()[0..1].try_into().unwrap();
                    let size_buffer: [u8; 4] = buffer.as_slice()[1..5].try_into().unwrap();
                    let id_buffer: [u8; 4] = buffer.as_slice()[5..9].try_into().unwrap();

                    let object_type = u8::from_le_bytes(type_buffer);
                    let object_size = u32::from_le_bytes(size_buffer);
                    let object_id = u32::from_le_bytes(id_buffer);

                    (object_type, object_size, object_id)
                };

                println!("Parsing HIRC object. type = {}, size = {}, id = {}", object_type, object_size, object_id);

                // Seek back to beginning of object
                data.seek(io::SeekFrom::Current(-9))?;

                // Buffer entire object the size indicated by the HIRC object in including
                // the object ID. There we only need to account for the type (byte) and 
                // size field (u32).
                let mut buffer = vec![0u8; (object_size + 5) as usize];
                data.read_exact(&mut buffer)?;

                // Attempt to parse object
                let (rest, object) = HIRCObject::from_bytes((buffer.as_slice(), 0))
                    .expect("Failed parsing object");

                println!("Parsed HIRC object: {:#?}", object);

                if !rest.0.is_empty() {
                    panic!("{} bytes left in buffer", rest.0.len());
                }
            }

            return Ok(());
        } else {
            data.seek(io::SeekFrom::Current(header.size as i64))?;
        }
    }

    Ok(())
}

#[derive(Debug, DekuRead, DekuWrite)]
struct SectionHeader {
    magic: [u8; 4],
    size: u32,
}

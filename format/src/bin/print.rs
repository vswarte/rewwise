use std::fs;
use std::env;
use std::path;
use std::io::Read;

fn main() {
    let args: Vec<String> = env::args().collect();

    for path in args[1..].iter() {
        let path = path::PathBuf::from(path);

        let mut handle = fs::File::open(&path)
            .expect("Could not acquire read file handle");

        let mut file_buffer = vec![];
        handle.read_to_end(&mut file_buffer)
            .expect("Could not read input file");

        let soundbank = wwise_format::parse_soundbank(&file_buffer)
            .expect("Could not parse bnk");

        println!("{:#?}", soundbank);
    }
}

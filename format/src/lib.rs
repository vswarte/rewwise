use deku::prelude::*;

mod bnk;
mod export;
mod serialization;

pub use bnk::*;
use export::PrepareExport;

pub fn parse_soundbank(bytes: &[u8]) -> Result<Soundbank, DekuError> {
    Soundbank::from_bytes((bytes, 0))
        .map(|r| r.1)
}

pub fn prepare_soundbank(soundbank: &mut Soundbank) {
    soundbank.prepare_export().unwrap();
}

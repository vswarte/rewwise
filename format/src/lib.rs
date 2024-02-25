use deku::prelude::*;

mod bnk;
mod export;
mod helper;
mod serialization;

pub use bnk::*;
pub use helper::*;

use export::PrepareExport;

pub fn parse_soundbank(bytes: &[u8]) -> Result<Soundbank, DekuError> {
    Soundbank::from_bytes((bytes, 0))
        .map(|r| r.1)
}

pub fn prepare_soundbank(soundbank: &mut Soundbank) {
    soundbank.prepare_export().unwrap();
}

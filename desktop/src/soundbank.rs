use std::sync;
use wwise_format::Soundbank;

pub static PRIMARY_SOUNDBANK: sync::OnceLock<sync::RwLock<Option<Soundbank>>> = sync::OnceLock::new();

pub fn init() {
    PRIMARY_SOUNDBANK.set(Default::default()).unwrap();
}

pub fn set(s: Soundbank) {
    *PRIMARY_SOUNDBANK.get()
        .unwrap()
        .write()
        .unwrap() = Some(s);
}

fn soundbank_read<'a>() -> sync::RwLockReadGuard<'a, Option<Soundbank>> {
    PRIMARY_SOUNDBANK.get()
        .unwrap()
        .read()
        .unwrap()
}

pub fn hirc(
    s: &wwise_format::Soundbank,
) -> Option<&wwise_format::HIRCSection> {
    let s = soundbank_read();
    for section in s.as_ref().unwrap().sections.iter() {
        match &section.body {
            wwise_format::SectionBody::HIRC(b) => return Some(b),
            _ => {},
        }
    }
    None
}

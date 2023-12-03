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

pub fn clear() {
    *PRIMARY_SOUNDBANK.get()
        .unwrap()
        .write()
        .unwrap() = None;
}

pub fn hirc(
    s: &wwise_format::Soundbank,
) -> Option<&wwise_format::HIRCSection> {
    for section in s.sections.iter() {
        if let wwise_format::SectionBody::HIRC(b) = &section.body {
            return Some(b)
        }
    }
    None
}

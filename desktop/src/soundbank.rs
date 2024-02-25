use std::sync;
use yew::prelude::*;
use wwise_format::{Soundbank, Section, HIRCObject};

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

pub fn has_soundbank() -> bool {
    PRIMARY_SOUNDBANK.get()
        .unwrap()
        .read()
        .unwrap()
        .is_some()
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

#[hook]
pub fn use_soundbank() -> sync::RwLockReadGuard<'static, Option<Soundbank>> {
    let lock = crate::soundbank::PRIMARY_SOUNDBANK.get()
        .expect("Could not acquire soundbank oncelock")
        .read()
        .expect("Could not acquire read lock on soundbank");

    lock
}

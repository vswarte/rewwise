use std::collections;

use crate::fnv;

pub type FNVDictionary = collections::HashMap<u32, String>;

pub fn parse_dictionary(input: &str) -> FNVDictionary {
    input.lines()
        .filter(|l| !l.is_empty() || !l.starts_with('#'))
        .map(|l| (fnv::create_hash(l), l.to_string()))
        .collect()
}

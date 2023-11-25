use std::num::Wrapping;

const FNV_BASE: Wrapping<u32> = Wrapping(2166136261);
const FNV_PRIME: Wrapping<u32> = Wrapping(16777619);

pub fn create_hash(input: &str) -> u32 {
    let input_lower = input.to_ascii_lowercase();
    let input_buffer = input_lower.as_bytes();

    let mut result = FNV_BASE;
    for byte in input_buffer {
        result *= FNV_PRIME;
        result ^= *byte as u32;
    }

    result.0
}

use core::str;
use std::str::Utf8Error;

pub fn i8_array_to_string(array: [i8; 32]) -> Result<String, Utf8Error> {
    // TODO i am not sure why, but the name field in the Nam event sometimes has some gibberish bytes
    // after the first null byte, so we can't just use from_utf8
    let end = array.iter().position(|&c| c == 0).unwrap_or(array.len());

    let bytes: Vec<u8> = array[..end].iter().map(|&c| c as u8).collect();
    str::from_utf8(&bytes).map(|s| s.to_string())
}

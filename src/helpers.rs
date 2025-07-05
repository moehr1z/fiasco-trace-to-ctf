use core::str;
use std::str::Utf8Error;

pub fn i8_array_to_string<const N: usize>(array: [i8; N]) -> Result<String, Utf8Error> {
    let end = array.iter().position(|&c| c == 0).unwrap_or(array.len());

    let bytes: Vec<u8> = array[..end].iter().map(|&c| c as u8).collect();

    if bytes.iter().any(|&b| !(b.is_ascii_graphic() || b == b' ')) {
        #[allow(invalid_from_utf8)]
        return Err(str::from_utf8(&[0xFF]).unwrap_err()); // fake UTF-8 error 
    }

    String::from_utf8(bytes).map_err(|e| e.utf8_error())
}

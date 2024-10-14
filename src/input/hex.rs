use hex::FromHexError;

pub fn is_valid_hex(data: &[u8]) -> bool {
    data.len() % 2 == 0 && data.iter().all(|&b| b.is_ascii_hexdigit())
}

pub fn hex_decode(input: Vec<u8>) -> Result<Vec<u8>, FromHexError> {
    Ok(hex::decode(input)?)
}
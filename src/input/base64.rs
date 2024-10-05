use base64::DecodeError;
use base64::engine::general_purpose;
use base64::prelude::*;

pub fn base64_decode(input: Vec<u8>) -> Result<Vec<u8>, DecodeError> {
    Ok(BASE64_STANDARD.decode(input.as_slice())?)
}

pub fn is_valid_base64(data: Vec<u8>) -> bool {
    if let Ok(decoded) = general_purpose::STANDARD.decode(data.clone()) {
        decoded != data
    } else {
        false
    }
}
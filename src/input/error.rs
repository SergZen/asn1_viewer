use std::{fmt, io};
use std::error::Error;
use base64::DecodeError;
use hex::FromHexError;

#[derive(Debug)]
pub enum InputError {
    IoError(io::Error),
    DecodeBase64Error(DecodeError),
    DecodeHexError(FromHexError),
    NoInputProvided,
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputError::IoError(e) => write!(f, "IO error: {}", e),
            InputError::DecodeBase64Error(e) => write!(f, "Base64 decode error: {}", e),
            InputError::DecodeHexError(e) => write!(f, "Hex decode error: {}", e),
            InputError::NoInputProvided => write!(f, "No input provided"),
        }
    }
}

impl Error for InputError {}

impl From<io::Error> for InputError {
    fn from(error: io::Error) -> Self {
        InputError::IoError(error)
    }
}

impl From<DecodeError> for InputError {
    fn from(error: DecodeError) -> Self {
        InputError::DecodeBase64Error(error)
    }
}

impl From<FromHexError> for InputError {
    fn from(error: FromHexError) -> Self {
        InputError::DecodeHexError(error)
    }
}
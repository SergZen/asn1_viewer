use std::fs::File;
use std::{fmt, io};
use std::error::Error;
use std::io::{BufRead, BufReader, IsTerminal, Read};
use std::path::{Path, PathBuf};
use ::base64::DecodeError;
use ::hex::FromHexError;
use clap::Parser;
use crate::input::base64::{base64_decode, is_valid_base64};
use crate::input::hex::{hex_decode, is_valid_hex};

pub mod base64;
pub mod hex;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the ASN.1 format file
    #[arg(short = 'f', long, value_name = "FILE")]
    file: Option<PathBuf>,

    /// ASN.1 specification string in base64 or hex
    #[arg(short = 'a', long)]
    asn1: Option<String>,
}

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

enum InputSource<'a> {
    File(&'a Path),
    Stdin,
}

fn read_content(source: InputSource) -> io::Result<Vec<u8>> {
    let buffer = match source {
        InputSource::File(path) => read_file(path)?,
        InputSource::Stdin => read_stdin()?,
    };

    let content = process_text_content(&buffer).unwrap_or_else(|_| buffer);

    Ok(content)
}

fn process_text_content(data: &[u8]) -> io::Result<Vec<u8>> {
    let reader = BufReader::new(data);
    let mut result = String::new();

    for line in reader.lines() {
        let line = line?;
        if !line.starts_with("----") {
            result.push_str(&line);
        }
    }

    Ok(Vec::from(result))
}

fn read_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn read_stdin() -> io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    io::stdin().lock().read_to_end(&mut buffer)?;

    Ok(buffer)
}

fn is_input_from_stdin() -> bool {
    !io::stdin().is_terminal()
}

pub(crate) fn get_input() -> Result<Vec<u8>, InputError> {
    let cli = Cli::parse();

    if let Some(asn1_base64) = cli.asn1 {
        Ok(Vec::from(asn1_base64))
    } else if let Some(file) = cli.file {
        Ok(read_content(InputSource::File(&file))?)
    } else if is_input_from_stdin() {
        Ok(read_content(InputSource::Stdin)?)
    } else {
        Err(InputError::NoInputProvided)
    }
}

pub(crate) fn get_input_data() -> Result<Vec<u8>, InputError> {
    let input = get_input()?;

    Ok(get_raw_data(input)?)
}

fn get_raw_data(input: Vec<u8>) -> Result<Vec<u8>, InputError> {
    if is_valid_base64(input.clone()) {
        return Ok(base64_decode(input)?);
    }

    if is_valid_hex(&input) {
        return Ok(hex_decode(input)?);
    }

    Ok(Vec::from(input))
}

use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, IsTerminal, Read};
use std::path::Path;
use clap::Parser;
use crate::cli::Cli;
use crate::input::base64::{base64_decode, is_valid_base64};
use crate::input::error::InputError;
use crate::input::hex::{hex_decode, is_valid_hex};

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
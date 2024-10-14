use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the ASN.1 format file
    #[arg(short = 'f', long, value_name = "FILE")]
    pub(crate) file: Option<PathBuf>,

    /// ASN.1 specification string in base64 or hex
    #[arg(short = 'a', long)]
    pub(crate) asn1: Option<String>,
}
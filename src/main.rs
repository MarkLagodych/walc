mod lambda;
mod wasm;

use anyhow::Result;
use clap::*;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Input WebAssembly module
    #[arg()]
    pub input_file: String,

    /// Output lambda calculus file
    #[arg(short = 'o')]
    pub output_file: String,
}

#[derive(thiserror::Error, Debug)]
enum IoError {
    #[error("Cannot read input file: {0}")]
    CannotReadInput(std::io::Error),
    #[error("Cannot write output file: {0}")]
    CannotWriteOutput(std::io::Error),
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    let source = std::fs::read(&args.input_file).map_err(IoError::CannotReadInput)?;

    let expr = crate::wasm::compile(&source)?;

    std::fs::write(&args.output_file, expr.to_string()).map_err(IoError::CannotWriteOutput)?;

    Ok(())
}

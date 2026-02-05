mod codegen;
mod walc;

use anyhow::{Result, anyhow};
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

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    let source =
        std::fs::read(&args.input_file).map_err(|e| anyhow!("Cannot read input file: {e}"))?;

    let expr = walc::compile_module(&source)?;

    std::fs::write(&args.output_file, expr.to_string())
        .map_err(|e| anyhow!("Cannot write output file: {e}"))?;

    Ok(())
}

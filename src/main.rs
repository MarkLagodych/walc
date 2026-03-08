mod analyzer;
mod codegen;

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
    let args = Args::parse();

    if let Err(e) = run(args) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let source =
        std::fs::read(&args.input_file).map_err(|e| anyhow!("Cannot read input file: {e}"))?;

    let expr = analyzer::compile(&source)?;

    std::fs::write(&args.output_file, expr.to_string())
        .map_err(|e| anyhow!("Cannot write output file: {e}"))?;

    Ok(())
}

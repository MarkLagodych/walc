mod analyzer;
mod codegen;

use anyhow::{Result, anyhow};
use clap::*;

use std::io::{Read, Write};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Input WebAssembly module. Use "-" for STDIN.
    #[arg()]
    pub input_file: String,

    /// Output lambda calculus file. Use "-" for STDOUT.
    #[arg(short = 'o', default_value = "-")]
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
    let source = if args.input_file == "-" {
        let mut buffer = Vec::new();
        std::io::stdin()
            .read_to_end(&mut buffer)
            .map_err(|e| anyhow!("Cannot read from STDIN: {e}"))?;

        buffer
    } else {
        std::fs::read(&args.input_file).map_err(|e| anyhow!("Cannot read input file: {e}"))?
    };

    let expr = analyzer::compile(&source)?;

    if args.output_file == "-" {
        std::io::stdout()
            .write_all(expr.to_string().as_bytes())
            .map_err(|e| anyhow!("Cannot write to STDOUT: {e}"))?;
    } else {
        std::fs::write(&args.output_file, expr.to_string())
            .map_err(|e| anyhow!("Cannot write output file: {e}"))?;
    }

    Ok(())
}

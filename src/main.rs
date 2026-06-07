mod analyzer;
mod codegen;

use anyhow::{Result, anyhow};
use clap::*;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Input WebAssembly module. Use "-" for STDIN.
    #[arg(default_value = "-")]
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
    let source =
        std::fs::read(&args.input_file).map_err(|e| anyhow!("Cannot read input file: {e}"))?;

    let expr = analyzer::compile(&source)?;

    if args.output_file == "-" {
        println!("{}", expr);
    } else {
        std::fs::write(&args.output_file, expr.to_string())
            .map_err(|e| anyhow!("Cannot write output file: {e}"))?;
    }

    Ok(())
}

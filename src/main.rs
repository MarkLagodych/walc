mod lambda;
mod wasm;

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

    let source = std::fs::read(&args.input_file).expect("Failed to read input file");

    let lambda = crate::wasm::compile(&source);

    std::fs::write(&args.output_file, lambda.to_string()).expect("Failed to write output file");
}

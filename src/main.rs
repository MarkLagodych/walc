mod lambda;
mod wasm;

use clap::*;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Input WebAssembly/WASI file
    #[arg()]
    pub input_file: String,

    /// Output lambda calculus file
    #[arg(short)]
    pub output_file: String,
}

fn main() {
    let args = Args::parse();

    let source = std::fs::read(&args.input_file).expect("Failed to read input file");

    let module = wasmbin::Module::decode_from(source.as_slice()).unwrap();

    let lambda = crate::wasm::compile(&module);

    std::fs::write(&args.output_file, lambda.to_string()).expect("Failed to write output file");
}

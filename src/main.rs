mod cli;
mod lambda;
mod wasm;

fn main() {
    let args = cli::parse_args();

    let source = std::fs::read(&args.input_file).expect("Failed to read input file");

    let module = wasm::Module::decode(&source);

    let lambda = lambda::compile_wasm(module);

    std::fs::write(&args.output_file, lambda.to_string()).expect("Failed to write output file");
}

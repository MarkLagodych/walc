mod cli;
mod lambda;
mod wasm;

fn main() {
    let args = cli::parse_args();

    let data = std::fs::read(&args.input_file).expect("Failed to read input file");

    let module = wasm::Module::parse(&data);
}

use wasmparser::*;

mod cli;
mod lambda;
mod wasm;

fn main() {
    let args = cli::parse_args();

    let data = std::fs::read(&args.input_file).expect("Failed to read input file");

    let parser = Parser::new(0);
    for payload in parser.parse_all(&data) {
        match payload.expect("Failed to parse payload") {
            Payload::Version {
                num,
                encoding,
                range,
            } => {
                assert!(
                    encoding == Encoding::Module,
                    "cannot parse WASM 2.0 components"
                );
                println!("WASM Version: {} at {:?}", num, range);
            }
            Payload::TypeSection(types) => {
                println!("Type Section:");
                for ty in types {
                    let ty = ty.expect("Failed to read type");
                    println!("  {:?}", ty);
                }
            }
            Payload::FunctionSection(functions) => {
                println!("Function Section:");
                for func in functions {
                    let func = func.expect("Failed to read function");
                    println!("  Type Index: {}", func);
                }
            }
            Payload::CodeSectionEntry(body) => {
                println!("Code Section Entry:");
                let reader = body
                    .get_operators_reader()
                    .expect("Failed to get operators reader");
                for op in reader {
                    let op = op.expect("Failed to read operator");
                    println!("  {:?}", op);
                }
            }
            _ => {}
        }
    }
}

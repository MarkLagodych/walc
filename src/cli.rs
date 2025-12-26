use clap::*;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Input WebAssembly/WASI file
    #[arg()]
    pub input_file: String,

    /// Output lambda calculus file
    #[arg(short)]
    pub output_file: Option<String>,
}

pub fn parse_args() -> Args {
    Args::parse()
}

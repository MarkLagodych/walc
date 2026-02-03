use crate::lambda;

use anyhow::{Result, anyhow};

use wasmparser as wasm;

pub fn compile(source: &[u8]) -> Result<lambda::Expr> {
    let mut parser = wasm::Parser::new(0);
    parser.set_features(wasm::WasmFeatures::LIME1);
    for payload in parser.parse_all(source) {
        match payload? {
            wasm::Payload::Version { encoding, .. } => match encoding {
                wasm::Encoding::Module => {}
                wasm::Encoding::Component => Err(anyhow!("WASM components are not supported"))?,
            },

            wasm::Payload::DataSection(section) => {}

            _ => {}
        }
    }

    let mut b = lambda::DefinitionBuilder::new();
    lambda::define_prelude(&mut b);
    let expr = b.build(lambda::walc_io::end());
    Ok(expr)
}

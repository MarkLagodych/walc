use crate::lambda;

use anyhow::{Result, anyhow};

use wasmparser::*;

pub fn compile(source: &[u8]) -> Result<lambda::Expr> {
    let mut parser = Parser::new(0);
    parser.set_features(WasmFeatures::LIME1);
    for payload in parser.parse_all(source) {
        match payload? {
            Payload::Version { encoding, .. } => match encoding {
                Encoding::Module => {}
                Encoding::Component => Err(anyhow!("WASM components are not supported"))?,
            },

            Payload::DataSection(section) => {}

            Payload::ImportSection(section) => {
                for import in section {
                    println!("import: {:?}", import?);
                }
            }

            _ => {}
        }
    }

    let mut b = lambda::DefinitionBuilder::new();
    lambda::define_prelude(&mut b);
    let expr = b.build(lambda::walc_io::end());
    Ok(expr)
}

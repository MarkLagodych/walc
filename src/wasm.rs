use wasmbin::{Module as Wasm, indices::*, sections::*};

#[derive(Debug)]
pub struct Module {
    pub main: FuncId,
    pub functions: Vec<FuncBody>,
}

impl Module {
    pub fn decode(wasm_source: &[u8]) -> Self {
        let module = Wasm::decode_from(wasm_source).unwrap();

        let mut functions = vec![];
        let mut main = None;

        for section in module.sections {
            match section {
                Section::Export(exports) => {
                    let exports = exports.try_contents().unwrap();

                    for export in exports {
                        if export.name == "main"
                            && let ExportDesc::Func(func_id) = export.desc
                        {
                            main = Some(func_id);
                        }
                    }
                }

                Section::Code(code_section) => {
                    let code_section = code_section.try_contents().unwrap();

                    for function_body in code_section {
                        let function_body = function_body.try_contents().unwrap().clone();
                        functions.push(function_body);
                    }
                }

                _ => {}
            }
        }

        Self {
            main: main.expect("no main function found"),
            functions,
        }
    }
}

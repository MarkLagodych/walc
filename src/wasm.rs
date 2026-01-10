pub struct Module {}

impl Module {
    pub fn decode(wasm_source: &[u8]) -> Self {
        let module = wasmbin::Module::decode_from(wasm_source).expect("Failed to parse WASM");

        Self {}
    }
}

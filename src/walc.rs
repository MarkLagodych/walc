use crate::codegen;

use anyhow::{Result, anyhow};

use wasmparser::*;

#[derive(Default)]
struct Compiler {
    defs: codegen::DefinitionBuilder,
    consts: codegen::number::ConstantStore,

    function_info: FunctionInfo,
    active_data_segment_infos: Vec<ActiveDataSegmentInfo>,
}

#[derive(Default)]
struct FunctionInfo {
    next_id: u32,
    walc_input_id: Option<u32>,
    walc_output_id: Option<u32>,
    walc_exit_id: Option<u32>,
    main_id: u32,
}

impl FunctionInfo {
    fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

struct ActiveDataSegmentInfo {
    data_segment_id: u32,
    offset_expr: codegen::Expr,
}

pub fn compile(source: &[u8]) -> Result<codegen::Expr> {
    let mut compiler = Compiler::new();

    let mut parser = Parser::new(0);
    parser.set_features(WasmFeatures::WASM1);

    for payload in parser.parse_all(source) {
        match payload? {
            Payload::Version { encoding, .. } => compiler.handle_version(encoding)?,
            Payload::ImportSection(section) => compiler.handle_imports(section)?,
            Payload::ExportSection(section) => compiler.handle_exports(section)?,
            Payload::DataSection(section) => compiler.handle_data(section)?,
            _ => {}
        }
    }

    Ok(compiler.result())
}

impl Compiler {
    fn new() -> Self {
        Self::default()
    }

    fn handle_version(&self, encoding: Encoding) -> Result<()> {
        match encoding {
            Encoding::Module => Ok(()),
            Encoding::Component => Err(anyhow!("WASM components are not supported")),
        }
    }

    fn handle_imports(&mut self, section: ImportSectionReader) -> Result<()> {
        for import in section.into_imports() {
            let import = import?;

            if import.module != "walc" {
                Err(anyhow!("Only imports from the 'walc' module are supported"))?
            }

            match import.ty {
                TypeRef::Func(_type_id) | TypeRef::FuncExact(_type_id) => {
                    // The types of the built-in functions are not checked for simplicity
                }
                _ => Err(anyhow!("Only function imports are supported"))?,
            }

            match import.name {
                "input" => {
                    self.function_info.walc_input_id = Some(self.function_info.next_id());
                }
                "output" => {
                    self.function_info.walc_output_id = Some(self.function_info.next_id());
                }
                "exit" => {
                    self.function_info.walc_exit_id = Some(self.function_info.next_id());
                }
                _ => Err(anyhow!("Unknown import: {}", import.name))?,
            }
        }

        Ok(())
    }

    fn handle_exports(&mut self, section: ExportSectionReader) -> Result<()> {
        for export in section {
            let export = export?;

            if export.name == "main" {
                self.function_info.main_id = export.index;
            }

            // Other exports are ignored because they will not be used in any way
        }

        Ok(())
    }

    fn handle_data(&mut self, section: DataSectionReader) -> Result<()> {
        for (data_segment_id, data_segment) in section.into_iter().enumerate() {
            let data_segment = data_segment?;

            self.defs.def(
                format!("DATA{data_segment_id}"),
                codegen::safe_list::from_bytes(&mut self.consts, data_segment.data),
            );

            if let DataKind::Active { offset_expr, .. } = data_segment.kind {
                let offset_expr = self.translate_const(&offset_expr)?;

                self.active_data_segment_infos.push(ActiveDataSegmentInfo {
                    data_segment_id: data_segment_id as u32,
                    offset_expr,
                });
            }
        }

        Ok(())
    }

    fn translate_const(&mut self, expr: &ConstExpr) -> Result<codegen::Expr> {
        let mut operators = expr.get_operators_reader().into_iter();

        let result = match operators
            .next()
            .ok_or(anyhow!("Empty constant expression"))??
        {
            Operator::I32Const { value } => self.consts.i32_const(value as u32),
            Operator::I64Const { value } => self.consts.i64_const(value as u64),
            Operator::F32Const { value } => self.consts.i32_const(value.bits()),
            Operator::F64Const { value } => self.consts.i64_const(value.bits()),
            op => Err(anyhow!("Unsupported constant expression: {:?}", op))?,
        };

        match operators
            .next()
            .ok_or(anyhow!("Constant expression missing end operator"))??
        {
            Operator::End => {}
            op => Err(anyhow!(
                "Unexpected operator in constant expression: {:?}",
                op
            ))?,
        }

        Ok(result)
    }

    fn result(self) -> codegen::Expr {
        let root = codegen::walc_io::end();

        let mut toplevel = codegen::DefinitionBuilder::new();
        codegen::define_prelude(&mut toplevel);
        self.consts.define_constants(&mut toplevel);
        toplevel.build(self.defs.build(root))
    }
}

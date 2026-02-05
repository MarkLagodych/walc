use crate::codegen;

use anyhow::{Result, anyhow};

use wasmparser::*;

type FuncId = u32;
type TypeId = u32;

#[derive(Default)]
struct Compiler {
    defs: codegen::DefinitionBuilder,
    consts: codegen::number::ConstantStore,

    function_info: FunctionInfo,
    active_data_segment_infos: Vec<ActiveDataSegmentInfo>,
}

#[derive(Default)]
struct FunctionInfo {
    next_id: FuncId,

    main_id: FuncId,
    walc_input_id: Option<FuncId>,
    walc_output_id: Option<FuncId>,
    walc_exit_id: Option<FuncId>,
    start_id: Option<FuncId>,

    type_infos: std::collections::HashMap<TypeId, FunctionTypeInfo>,
    /// Indexed by function IDs
    function_types: Vec<TypeId>,
}

impl FunctionInfo {
    fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn get_function_type_info(&self, func_id: FuncId) -> FunctionTypeInfo {
        let type_id = self.function_types[func_id as usize];
        self.type_infos[&type_id].clone()
    }
}

#[derive(Clone)]

struct FunctionTypeInfo {
    param_count: usize,
    result_count: usize,
}

struct ActiveDataSegmentInfo {
    data_segment_id: u32,
    offset_expr: codegen::Expr,
}

const SUPPORTED_FEATURES: WasmFeatures = WasmFeatures::WASM1;

pub fn compile_module(source: &[u8]) -> Result<codegen::Expr> {
    Validator::new_with_features(SUPPORTED_FEATURES)
        .validate_all(source)
        .map_err(|e| anyhow!("failed to validate the module: {}", e))?;

    let mut compiler = Compiler::new();
    compiler.parse_module(source)?;
    Ok(compiler.assemble())
}

/// Retrieves the underlying representation for a WASM value type.
fn val_type_repr(val_type: ValType) -> Result<codegen::op::ValueRepr> {
    use codegen::op::ValueRepr;
    match val_type {
        ValType::I32 | ValType::F32 => Ok(ValueRepr::I32),
        ValType::I64 | ValType::F64 => Ok(ValueRepr::I64),
        _ => Err(anyhow!("Unsupported local type: {:?}", val_type)),
    }
}

impl Compiler {
    fn new() -> Self {
        Self::default()
    }

    fn parse_module(&mut self, source: &[u8]) -> Result<()> {
        let mut parser = Parser::new(0);
        parser.set_features(SUPPORTED_FEATURES);

        for payload in parser.parse_all(source) {
            match payload? {
                Payload::TypeSection(section) => self.handle_types(section)?,
                Payload::ImportSection(section) => self.handle_imports(section)?,
                Payload::FunctionSection(section) => self.handle_function_types(section)?,
                Payload::TableSection(section) => self.handle_table(section)?,
                Payload::GlobalSection(section) => self.handle_globals(section)?,
                Payload::ExportSection(section) => self.handle_exports(section)?,
                Payload::StartSection { func, .. } => self.handle_start(func)?,
                Payload::ElementSection(section) => self.handle_elements(section)?,
                Payload::CodeSectionEntry(function) => self.handle_function(function)?,
                Payload::DataSection(section) => self.handle_data(section)?,

                Payload::Version { encoding, .. } => {
                    if matches!(encoding, Encoding::Component) {
                        Err(anyhow!("WASM components are not supported"))?
                    }
                }

                Payload::CodeSectionStart { .. } => {
                    // This section start marker is ignored because it only contains
                    // the function count, which is not important
                }

                Payload::MemorySection(_section) => {
                    // This section is ignored because:
                    // 1. WASM 1.0 modules can only have one memory
                    // 2. WALC memory is lazy and always has the size of 4 GiB,
                    //    so the initial memory size is irrelevant
                }

                payload => Err(anyhow!("Unsupported section: {:?}", payload))?,
            }
        }

        Ok(())
    }

    fn handle_imports(&mut self, section: ImportSectionReader) -> Result<()> {
        for import in section.into_imports() {
            let import = import?;

            if import.module != "walc" {
                Err(anyhow!("Only imports from the 'walc' module are allowed"))?
            }

            match import.ty {
                TypeRef::Func(type_id) | TypeRef::FuncExact(type_id) => {
                    // TODO check built-in function types?

                    // Imported functions' types must be recorded so that indexes of the function
                    // type list will match function IDs
                    self.function_info.function_types.push(type_id);
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
                _ => Err(anyhow!(
                    "Unknown import: {} (only 'input', 'output', and 'exit' are allowed)",
                    import.name
                ))?,
            }
        }

        Ok(())
    }

    fn handle_start(&mut self, func: u32) -> Result<()> {
        self.function_info.start_id = Some(func);
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
                codegen::list::from_bytes(&mut self.consts, data_segment.data),
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

        let result = match operators.next().unwrap()? {
            Operator::I32Const { value } => self.consts.i32_const(value as u32),
            Operator::I64Const { value } => self.consts.i64_const(value as u64),
            Operator::F32Const { value } => self.consts.i32_const(value.bits()),
            Operator::F64Const { value } => self.consts.i64_const(value.bits()),
            op => Err(anyhow!("Unsupported constant expression: {:?}", op))?,
        };

        match operators.next().unwrap()? {
            Operator::End => {}

            op => Err(anyhow!(
                "Unexpected operator in constant expression: {:?}",
                op
            ))?,
        }

        Ok(result)
    }

    fn read_function_types(section: TypeSectionReader) -> impl Iterator<Item = Result<FuncType>> {
        section
            .into_iter()
            .map(|recursive_type_group| -> Result<FuncType> {
                let mut subtypes = recursive_type_group?.into_types();
                let first = subtypes.nth(0).unwrap();
                match first.composite_type.inner {
                    CompositeInnerType::Func(func_type) => Ok(func_type),
                    _ => Err(anyhow!("Only function types are supported")),
                }
            })
    }

    fn handle_types(&mut self, section: TypeSectionReader) -> Result<()> {
        for (type_id, ty) in Self::read_function_types(section).enumerate() {
            let ty = ty?;

            self.function_info.type_infos.insert(
                type_id as TypeId,
                FunctionTypeInfo {
                    param_count: ty.params().len(),
                    result_count: ty.results().len(),
                },
            );
        }

        Ok(())
    }

    fn handle_function_types(&mut self, section: FunctionSectionReader) -> Result<()> {
        for type_id in section.into_iter() {
            let type_id = type_id?;

            self.function_info.function_types.push(type_id);
        }

        Ok(())
    }

    fn get_function_local_reprs(
        &mut self,
        func: &FunctionBody,
    ) -> Result<Vec<codegen::op::ValueRepr>> {
        let mut local_reprs = Vec::<codegen::op::ValueRepr>::new();

        for local_declaration in func.get_locals_reader()?.into_iter() {
            let (count, val_type) = local_declaration?;
            local_reprs.extend(std::iter::repeat_n(
                val_type_repr(val_type)?,
                count as usize,
            ));
        }

        Ok(local_reprs)
    }

    fn handle_function(&mut self, func: FunctionBody) -> Result<()> {
        let func_id = self.function_info.next_id();

        let func_type_info = self.function_info.get_function_type_info(func_id);

        let mut function_builder = codegen::op::FunctionBuilder::new(
            func_type_info.param_count,
            func_type_info.result_count,
            &self.get_function_local_reprs(&func)?,
        );

        for op in func.get_operators_reader()?.into_iter() {
            let op = op?;

            // TODO read operators
        }

        Ok(())
    }

    fn handle_table(&mut self, _section: TableSectionReader) -> Result<()> {
        // TODO
        Ok(())
    }

    fn handle_globals(&mut self, _section: GlobalSectionReader) -> Result<()> {
        // TODO
        Ok(())
    }

    fn handle_elements(&mut self, _section: ElementSectionReader) -> Result<()> {
        // TODO
        Ok(())
    }

    fn assemble(self) -> codegen::Expr {
        // TODO
        let root_expr = codegen::walc_io::end();

        let mut toplevel = codegen::DefinitionBuilder::new();
        codegen::define_prelude(&mut toplevel);
        self.consts.define_constants(&mut toplevel);
        toplevel.build(self.defs.build(root_expr))
    }
}

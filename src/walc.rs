use crate::codegen;

use anyhow::{Result, anyhow};

use wasmparser::*;

type FuncId = u32;
type TypeId = u32;

#[derive(Default)]
struct Module {
    defs: codegen::DefinitionBuilder,
    consts: codegen::number::ConstantStore,

    function_info: FunctionInfo,
    active_data_segment_infos: Vec<ActiveDataSegmentInfo>,
}

#[derive(Default)]
struct FunctionInfo {
    next_id: FuncId,

    main_id: Option<FuncId>,
    walc_input_id: Option<FuncId>,
    walc_output_id: Option<FuncId>,
    walc_exit_id: Option<FuncId>,
    start_id: Option<FuncId>,

    /// Indexed by TypeId
    types: Vec<FuncType>,
    /// Indexed by FuncId
    type_map: Vec<TypeId>,
}

impl FunctionInfo {
    fn next_id(&mut self) -> FuncId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn get_type(&self, func_id: FuncId) -> &FuncType {
        let type_id = self.type_map[func_id as usize];
        &self.types[type_id as usize]
    }
}

struct ActiveDataSegmentInfo {
    data_segment_id: u32,
    offset_expr: codegen::Expr,
}

/// Maximum ID for functions, globals, and locals
const MAX_ID: u32 = u16::MAX as u32;
/// Maximum count for functions, globals, and locals
const MAX_COUNT: u32 = MAX_ID + 1;

const SUPPORTED_FEATURES: WasmFeatures = WasmFeatures::WASM1;

pub fn compile_module(source: &[u8]) -> Result<codegen::Expr> {
    Validator::new_with_features(SUPPORTED_FEATURES)
        .validate_all(source)
        .map_err(|e| anyhow!("failed to validate the module: {}", e))?;

    let mut module = Module::new();
    module.parse(source)?;
    module.assemble()
}

impl Module {
    fn new() -> Self {
        Self::default()
    }

    fn parse(&mut self, source: &[u8]) -> Result<()> {
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

                // Memory section is ignored because:
                // - WASM 1.0 modules can only have one memory
                // - WALC memory is lazy and always has a virtual size of 4 GiB,
                //   so the initial memory size is irrelevant
                Payload::MemorySection(_section) => {}

                // Other stuff is checked by the validator
                _ => {}
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

            let type_id = match import.ty {
                TypeRef::Func(type_id) | TypeRef::FuncExact(type_id) => type_id,
                _ => Err(anyhow!("Only function imports are supported"))?,
            };

            // Imported functions' types must be recorded so that indexes of the function
            // type list will match function IDs
            self.function_info.type_map.push(type_id);

            let func_id = self.function_info.next_id();

            let func_type = self.function_info.types.get(type_id as usize).unwrap();

            match import.name {
                "input" => {
                    if !(func_type.params().is_empty() && func_type.results() == [ValType::I32]) {
                        Err(anyhow!("walc.input must have type () -> (i32)"))?
                    }

                    self.function_info.walc_input_id = Some(func_id);
                }
                "output" => {
                    if !(func_type.params() == [ValType::I32] && func_type.results().is_empty()) {
                        Err(anyhow!("walc.output must have type (i32) -> ()"))?
                    }

                    self.function_info.walc_output_id = Some(func_id);
                }
                "exit" => {
                    if !(func_type.params().is_empty() && func_type.results().is_empty()) {
                        Err(anyhow!("walc.exit must have type () -> ()"))?
                    }

                    self.function_info.walc_exit_id = Some(func_id);
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
                self.function_info.main_id = Some(export.index);

                let func_type = self.function_info.get_type(export.index);

                if !(func_type.params().is_empty() && func_type.results().is_empty()) {
                    Err(anyhow!("'main' must have type () -> ()"))?
                }
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
        if section.count() > MAX_COUNT {
            Err(anyhow!(
                "Too many functions: {} (max is {})",
                section.count(),
                MAX_COUNT
            ))?
        }

        for func_type in Self::read_function_types(section) {
            self.function_info.types.push(func_type?);
        }

        Ok(())
    }

    fn handle_function_types(&mut self, section: FunctionSectionReader) -> Result<()> {
        for type_id in section.into_iter() {
            self.function_info.type_map.push(type_id?);
        }

        Ok(())
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

    fn get_function_local_reprs(
        &mut self,
        func: &FunctionBody,
    ) -> Result<Vec<codegen::op::ValueRepr>> {
        let mut local_reprs = Vec::<codegen::op::ValueRepr>::new();

        for local_declaration in func.get_locals_reader()?.into_iter() {
            let (count, val_type) = local_declaration?;
            local_reprs.extend(std::iter::repeat_n(
                Self::val_type_repr(val_type)?,
                count as usize,
            ));
        }

        Ok(local_reprs)
    }

    fn handle_function(&mut self, func: FunctionBody) -> Result<()> {
        let func_id = self.function_info.next_id();

        let local_reprs = self.get_function_local_reprs(&func)?;

        if local_reprs.len() > (MAX_COUNT as usize) {
            Err(anyhow!(
                "Too many locals in function {}: {} (max is {})",
                func_id,
                local_reprs.len(),
                MAX_COUNT
            ))?;
        }

        let func_type = self.function_info.get_type(func_id);

        let mut func_builder = codegen::op::FunctionBuilder::new(
            func_type.params().len(),
            func_type.results().len(),
            &local_reprs,
        );

        for op in func.get_operators_reader()?.into_iter() {
            let op = op?;

            // TODO read operators
        }

        Ok(())
    }

    fn handle_table(&mut self, section: TableSectionReader) -> Result<()> {
        // TODO
        Ok(())
    }

    fn handle_globals(&mut self, section: GlobalSectionReader) -> Result<()> {
        if section.count() > MAX_COUNT {
            Err(anyhow!(
                "Too many globals: {} (max is {})",
                section.count(),
                MAX_COUNT
            ))?;
        }

        for global in section.into_iter() {
            let global = global?;

            let init_expr = self.translate_const(&global.init_expr)?;

            // TODO
        }

        Ok(())
    }

    fn handle_elements(&mut self, section: ElementSectionReader) -> Result<()> {
        // TODO
        Ok(())
    }

    fn assemble(self) -> Result<codegen::Expr> {
        if self.function_info.main_id.is_none() {
            Err(anyhow!("The module must export a 'main' function"))?
        }

        if let Some(start_id) = self.function_info.start_id {
            // TODO
        }

        // TODO handle active data segments

        // TODO
        let root_expr = codegen::walc_io::end();

        let mut toplevel = codegen::DefinitionBuilder::new();
        toplevel.define_prelude();
        self.consts.define_constants(&mut toplevel);
        let expr = toplevel.build(self.defs.build(root_expr));

        Ok(expr)
    }
}

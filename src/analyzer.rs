use anyhow::{Result, anyhow};

use crate::codegen;

use wasmparser::*;
pub use wasmparser::{Operator, ValType};

pub type FuncId = u32;
pub type DataId = u32;
type TypeId = u32;

type IdCounter = std::ops::RangeFrom<u32>;

pub struct Analyzer {
    mod_builder: codegen::program::ProgramBuilder,

    function_id: IdCounter,
    data_id: IdCounter,

    /// Indexed by TypeId
    function_types: Vec<FuncType>,
    /// Indexed by FuncId
    function_type_map: Vec<TypeId>,

    has_main: bool,
}

impl Analyzer {
    /// Maximum ID for functions, globals, and locals
    const MAX_ID: u32 = u16::MAX as u32;
    const MAX_COUNT: u32 = Self::MAX_ID + 1;

    const SUPPORTED_FEATURES: WasmFeatures = WasmFeatures::WASM1;

    pub fn new() -> Self {
        Self {
            mod_builder: codegen::program::ProgramBuilder::new(),
            function_id: 0..,
            data_id: 0..,
            function_types: Vec::new(),
            function_type_map: Vec::new(),
            has_main: false,
        }
    }

    pub fn compile(mut self, source: &[u8]) -> Result<codegen::Expr> {
        Validator::new_with_features(Self::SUPPORTED_FEATURES)
            .validate_all(source)
            .map_err(|e| anyhow!("validation failed: {e}"))?;

        let mut parser = Parser::new(0);
        parser.set_features(Self::SUPPORTED_FEATURES);
        for payload in parser.parse_all(source) {
            self.handle_payload(payload?)?;
        }

        if !self.has_main {
            Err(anyhow!("The module does not export a 'main' function"))?
        }

        Ok(self.mod_builder.build())
    }

    fn handle_payload(&mut self, payload: Payload<'_>) -> Result<()> {
        match payload {
            Payload::TypeSection(section) => self.handle_types(section)?,
            Payload::ImportSection(section) => self.handle_imports(section)?,
            Payload::FunctionSection(section) => self.handle_function_types(section)?,
            Payload::TableSection(section) => self.handle_tables(section)?,
            Payload::GlobalSection(section) => self.handle_globals(section)?,
            Payload::ExportSection(section) => self.handle_exports(section)?,
            Payload::StartSection { func, .. } => self.handle_start(func)?,
            Payload::ElementSection(section) => self.handle_elements(section)?,
            Payload::CodeSectionEntry(function) => self.handle_function(function)?,
            Payload::DataSection(section) => self.handle_data(section)?,

            // Memory section is ignored because WASM 1.0 modules can only have one memory
            // and its size properties are irrelevant for WALC because WALC memory
            // is lazy and always has a virtual size of 4 GiB.
            Payload::MemorySection(_section) => {}

            // Other sections are checked by the validator
            _ => {}
        }

        Ok(())
    }

    fn get_function_type(&self, func_id: FuncId) -> &FuncType {
        let type_id = self.function_type_map[func_id as usize];
        &self.function_types[type_id as usize]
    }

    fn handle_imports(&mut self, section: ImportSectionReader) -> Result<()> {
        for import in section.into_imports() {
            let import = import?;

            let type_id = match import.ty {
                TypeRef::Func(type_id) | TypeRef::FuncExact(type_id) => type_id,
                _ => Err(anyhow!("Only function imports are supported"))?,
            };

            // Imported functions' types must be recorded so that indexes of function_types
            // will match function IDs
            self.function_type_map.push(type_id);

            let func_id = self.function_id.next().unwrap();

            let func_type = &self.function_types[type_id as usize];

            match import.name {
                "input" => {
                    if !(func_type.params().is_empty() && func_type.results() == [ValType::I32]) {
                        Err(anyhow!("walc.input must have type () -> (i32)"))?
                    }
                }
                "output" => {
                    if !(func_type.params() == [ValType::I32] && func_type.results().is_empty()) {
                        Err(anyhow!("walc.output must have type (i32) -> ()"))?
                    }
                }
                "exit" => {
                    if !(func_type.params().is_empty() && func_type.results().is_empty()) {
                        Err(anyhow!("walc.exit must have type () -> ()"))?
                    }
                }
                name => Err(anyhow!(
                    "Unknown import: {name} (only 'input', 'output', and 'exit' are supported)",
                ))?,
            }

            self.mod_builder.handle_import(import.name, func_id);
        }

        Ok(())
    }

    fn handle_exports(&mut self, section: ExportSectionReader) -> Result<()> {
        for export in section {
            let export = export?;

            if export.name != "main" {
                // Other exports are ignored because they will not be used in any way
                continue;
            }

            let func_type = self.get_function_type(export.index);

            if !(func_type.params().is_empty() && func_type.results().is_empty()) {
                Err(anyhow!("'main' must have type () -> ()"))?
            }

            self.has_main = true;

            self.mod_builder.handle_main(export.index);
        }

        Ok(())
    }

    fn handle_start(&mut self, func: u32) -> Result<()> {
        self.mod_builder.handle_start(func);
        Ok(())
    }

    fn handle_data(&mut self, section: DataSectionReader) -> Result<()> {
        for data_segment in section.into_iter() {
            let data_segment = data_segment?;

            let data_id = self.data_id.next().unwrap();

            let mut active_offset = None;

            if let DataKind::Active { offset_expr, .. } = data_segment.kind {
                active_offset = match offset_expr.get_operators_reader().read()? {
                    Operator::I32Const { value } => Some(value as u32),
                    // Validator must ensure that offset_expr is I32
                    _ => unreachable!(),
                };
            }

            self.mod_builder
                .handle_data(data_id, data_segment.data, active_offset);
        }

        Ok(())
    }

    fn handle_types(&mut self, section: TypeSectionReader) -> Result<()> {
        for func_type in section
            .into_iter()
            .map(|recursive_type_group| -> Result<FuncType> {
                let mut subtypes = recursive_type_group?.into_types();
                let first = subtypes.nth(0).unwrap();
                match first.composite_type.inner {
                    CompositeInnerType::Func(func_type) => Ok(func_type),
                    _ => Err(anyhow!("Only function types are supported")),
                }
            })
        {
            self.function_types.push(func_type?);
        }

        Ok(())
    }

    fn handle_function_types(&mut self, section: FunctionSectionReader) -> Result<()> {
        if section.count() > Self::MAX_COUNT {
            Err(anyhow!(
                "Too many functions: {} (max is {})",
                section.count(),
                Self::MAX_COUNT
            ))?
        }

        for type_id in section.into_iter() {
            self.function_type_map.push(type_id?);
        }

        Ok(())
    }

    fn read_function_local_types(func: &FunctionBody) -> Result<Vec<ValType>> {
        let mut local_types = Vec::<ValType>::new();

        for local_declaration in func.get_locals_reader()?.into_iter() {
            let (count, val_type) = local_declaration?;
            local_types.extend(std::iter::repeat_n(val_type, count as usize));
        }

        Ok(local_types)
    }

    fn handle_function(&mut self, func: FunctionBody) -> Result<()> {
        let func_id = self.function_id.next().unwrap();

        let local_types = Self::read_function_local_types(&func)?;

        if local_types.len() > (Self::MAX_COUNT as usize) {
            Err(anyhow!(
                "Too many locals in function {}: {} (max is {})",
                func_id,
                local_types.len(),
                Self::MAX_COUNT
            ))?;
        }

        let func_type = self.get_function_type(func_id);
        let param_count = func_type.params().len() as u32;
        // There is at most one result due to validation
        let has_result = !func_type.results().is_empty();

        let operators = func
            .get_operators_reader()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        self.mod_builder.handle_function(
            func_id,
            param_count,
            has_result,
            &local_types,
            &operators,
        );

        Ok(())
    }

    fn handle_globals(&mut self, section: GlobalSectionReader) -> Result<()> {
        if section.count() > Self::MAX_COUNT {
            Err(anyhow!(
                "Too many globals: {} (max is {})",
                section.count(),
                Self::MAX_COUNT
            ))?;
        }

        for global in section.into_iter() {
            let global = global?;

            let init = match global.init_expr.get_operators_reader().read()? {
                Operator::I32Const { value } => value as u64,
                Operator::I64Const { value } => value as u64,
                Operator::F32Const { value } => value.bits() as u64,
                Operator::F64Const { value } => value.bits(),
                // Other operators are unsupported in WASM 1.0
                _ => unreachable!(),
            };

            // The mutability flag is ignored because all globals are mutable in WALC
            let ty = global.ty.content_type;

            self.mod_builder.handle_global(ty, init);
        }

        Ok(())
    }

    fn handle_tables(&mut self, section: TableSectionReader) -> Result<()> {
        for table in section {
            let table = table?;

            self.mod_builder.handle_table(table.ty.initial as u32);
        }

        Ok(())
    }

    fn handle_elements(&mut self, section: ElementSectionReader) -> Result<()> {
        for element in section {
            let element = element?;

            if let ElementKind::Active { offset_expr, .. } = element.kind {
                let offset = if let Operator::I32Const { value } =
                    offset_expr.get_operators_reader().read()?
                {
                    value as u32
                } else {
                    // Offsets are always I32 in WASM 1.0
                    unreachable!()
                };

                let items = if let ElementItems::Functions(funcs) = element.items {
                    funcs
                } else {
                    // Other items are not supported in WASM 1.0
                    unreachable!()
                };

                let functions = items.into_iter().collect::<Result<Vec<_>, _>>()?;

                self.mod_builder.handle_elements(offset, &functions);
            }

            // Other element kinds do not exist in WASM 1.0
        }

        Ok(())
    }
}

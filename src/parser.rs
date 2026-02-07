//! Parses and validates WASM and type-checks functions that are important for WALC.

use crate::analyzer::Analyzer;

use anyhow::{Result, anyhow};

pub use wasmparser::*;

pub type FuncId = u32;
pub type DataId = u32;
pub type TypeId = u32;
pub type GlobalId = u32;
pub type TableId = u32;

pub struct Parser<'a> {
    source: &'a [u8],

    analyzer: &'a mut Analyzer,

    next_function_id: FuncId,

    /// Indexed by TypeId
    function_types: Vec<FuncType>,
    /// Indexed by FuncId
    function_type_map: Vec<TypeId>,

    has_main: bool,

    next_data_id: DataId,
}

impl<'a> Parser<'a> {
    /// Maximum ID for functions, globals, and locals
    const MAX_ID: u32 = u16::MAX as u32;
    const MAX_COUNT: u32 = Self::MAX_ID + 1;

    const SUPPORTED_FEATURES: WasmFeatures = WasmFeatures::WASM1;

    pub fn new(source: &'a [u8], analyzer: &'a mut Analyzer) -> Self {
        Self {
            source,
            analyzer,
            next_function_id: 0,
            next_data_id: 0,
            function_types: Vec::new(),
            function_type_map: Vec::new(),
            has_main: false,
        }
    }

    pub fn parse(mut self) -> Result<()> {
        Validator::new_with_features(Self::SUPPORTED_FEATURES)
            .validate_all(self.source)
            .map_err(|e| anyhow!("validation failed: {e}"))?;

        let mut parser = wasmparser::Parser::new(0);
        parser.set_features(Self::SUPPORTED_FEATURES);

        for payload in parser.parse_all(self.source) {
            match payload? {
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

                // Memory section is ignored because:
                // - WASM 1.0 modules can only have one memory
                // - WALC memory is lazy and always has a virtual size of 4 GiB,
                //   so the initial memory size is irrelevant
                Payload::MemorySection(_section) => {}

                // Other sections are checked by the validator
                _ => {}
            }
        }

        if !self.has_main {
            Err(anyhow!("The module does not export a 'main' function"))?
        }

        Ok(())
    }

    fn next_function_id(&mut self) -> FuncId {
        let id = self.next_function_id;
        self.next_function_id += 1;
        id
    }

    fn next_data_id(&mut self) -> DataId {
        let id = self.next_data_id;
        self.next_data_id += 1;
        id
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

            let func_id = self.next_function_id();

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

            self.analyzer.handle_import(import.name, func_id);
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

            self.analyzer.handle_main(export.index);
        }

        Ok(())
    }

    fn handle_start(&mut self, func: u32) -> Result<()> {
        self.analyzer.handle_start(func);
        Ok(())
    }

    fn handle_data(&mut self, section: DataSectionReader) -> Result<()> {
        for data_segment in section.into_iter() {
            let data_segment = data_segment?;

            let data_id = self.next_data_id();

            let mut active_offset = None;

            if let DataKind::Active { offset_expr, .. } = data_segment.kind {
                active_offset = match offset_expr.get_operators_reader().read()? {
                    Operator::I32Const { value } => Some(value as u32),
                    // Validator must ensure that offset_expr is I32
                    _ => unreachable!(),
                };
            }

            self.analyzer
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
        let func_id = self.next_function_id();

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

        let operators = func
            .get_operators_reader()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        self.analyzer
            .handle_function(func_id, param_count, &local_types, &operators);

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

            // All globals can be mutable in WALC

            // TODO how to represent a variant of I32/I64/F32/F64 for the handler?
            // let init_expr = self.expr_to_u32(&global.init_expr)?;

            // TODO
        }

        Ok(())
    }

    fn handle_tables(&mut self, section: TableSectionReader) -> Result<()> {
        // TODO
        Ok(())
    }

    fn handle_elements(&mut self, section: ElementSectionReader) -> Result<()> {
        // TODO
        Ok(())
    }
}

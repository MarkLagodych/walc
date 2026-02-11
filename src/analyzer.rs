use anyhow::{Result, anyhow};

use crate::codegen;

use wasmparser::*;
pub use wasmparser::{FuncType, Operator, ValType};

pub type FuncId = u32;
pub type TypeId = u32;
pub type DataSegmentId = u32;

pub struct FunctionInfo<'a> {
    pub function_types: &'a [FuncType],
    pub function_type: &'a FuncType,
    pub local_types: &'a [ValType],
    pub instructions: &'a [Operator<'a>],
}

#[derive(Default)]
pub struct Analyzer {
    prog_builder: codegen::program::ProgramBuilder,

    /// Indexed by TypeId
    function_types: Vec<FuncType>,

    /// Stack of function type IDs.
    /// The last type ID corresponds to the first function body defined in the module.
    function_type_ids: Vec<TypeId>,
}

impl Analyzer {
    /// Maximum ID for functions, globals, and locals
    const MAX_ID: u32 = u16::MAX as u32;
    const MAX_COUNT: u32 = Self::MAX_ID + 1;

    const SUPPORTED_FEATURES: WasmFeatures = WasmFeatures::WASM1;

    pub fn new() -> Self {
        Self::default()
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

        Ok(self.prog_builder.build())
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

    fn handle_imports(&mut self, section: ImportSectionReader) -> Result<()> {
        for import in section.into_imports() {
            let import = import?;

            if import.module != "walc" {
                Err(anyhow!(
                    "Unknown import module: '{}', only 'walc' is supported",
                    import.module
                ))?
            }

            let type_id = match import.ty {
                TypeRef::Func(type_id) | TypeRef::FuncExact(type_id) => type_id,
                _ => Err(anyhow!("Only function imports are supported"))?,
            };

            let func_type = &self.function_types[type_id as usize];

            match import.name {
                "input" => {
                    if !(func_type.params().is_empty() && func_type.results() == [ValType::I32]) {
                        Err(anyhow!("'walc.input' must have type () -> (i32)"))?
                    }
                }
                "output" => {
                    if !(func_type.params() == [ValType::I32] && func_type.results().is_empty()) {
                        Err(anyhow!("'walc.output' must have type (i32) -> ()"))?
                    }
                }
                "exit" => {
                    if !(func_type.params().is_empty() && func_type.results().is_empty()) {
                        Err(anyhow!("'walc.exit' must have type () -> ()"))?
                    }
                }
                name => Err(anyhow!(
                    "Unknown import: '{name}' (only 'input', 'output', and 'exit' are supported)",
                ))?,
            }

            self.prog_builder.handle_import(import.name);
        }

        Ok(())
    }

    fn handle_exports(&mut self, section: ExportSectionReader) -> Result<()> {
        let mut has_main = false;

        for export in section {
            let export = export?;

            if export.name != "main" {
                // Other exports are ignored because they will not be used in any way
                continue;
            }

            let len = self.function_types.len();
            let func_type = &self.function_types[len - 1 - export.index as usize];

            if !(func_type.params().is_empty() && func_type.results().is_empty()) {
                Err(anyhow!("'main' must have type () -> ()"))?
            }

            has_main = true;

            self.prog_builder.handle_main(export.index);
        }

        if !has_main {
            Err(anyhow!("The module does not export a 'main' function"))?
        }

        Ok(())
    }

    fn handle_start(&mut self, func: u32) -> Result<()> {
        self.prog_builder.handle_start(func);
        Ok(())
    }

    fn handle_data(&mut self, section: DataSectionReader) -> Result<()> {
        for (data_id, data_segment) in section.into_iter().enumerate() {
            let data_segment = data_segment?;

            let target_memory_offsest = match data_segment.kind {
                DataKind::Active { offset_expr, .. } => {
                    match offset_expr.get_operators_reader().read()? {
                        Operator::I32Const { value } => value as u32,
                        // Validator must ensure that offset_expr is I32
                        _ => unreachable!(),
                    }
                }
                // Passive data segments are not supported in WASM 1.0
                _ => unreachable!(),
            };

            self.prog_builder
                .handle_data(data_id as u32, data_segment.data, target_memory_offsest);
        }

        Ok(())
    }

    fn handle_types(&mut self, section: TypeSectionReader) -> Result<()> {
        let func_types = section.into_iter().map(|type_group| -> Result<FuncType> {
            let func_type = type_group?
                .into_types()
                .next()
                .unwrap()
                .unwrap_func()
                .clone();

            Ok(func_type)
        });

        for func_type in func_types {
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

        self.function_type_ids = section.into_iter().collect::<Result<Vec<_>, _>>()?;

        self.function_type_ids.reverse();

        Ok(())
    }

    fn read_function_local_types(func: &FunctionBody) -> Result<Vec<ValType>> {
        let mut local_types = Vec::<ValType>::new();

        for local_declaration in func.get_locals_reader()?.into_iter() {
            let (count, val_type) = local_declaration?;
            local_types.extend(std::iter::repeat_n(val_type, count as usize));
        }

        if local_types.len() > (Self::MAX_COUNT as usize) {
            Err(anyhow!(
                "Too many locals in a function: {} (max is {})",
                local_types.len(),
                Self::MAX_COUNT
            ))?;
        }

        Ok(local_types)
    }

    fn handle_function(&mut self, func: FunctionBody) -> Result<()> {
        let type_id = self.function_type_ids.pop().unwrap();
        let func_type = &self.function_types[type_id as usize];

        let local_types = Self::read_function_local_types(&func)?;

        let instructions = func
            .get_operators_reader()?
            .into_iter()
            .collect::<Result<Vec<Operator>, _>>()?;

        self.prog_builder.handle_function(&FunctionInfo {
            function_types: &self.function_types,
            function_type: func_type,
            local_types: &local_types,
            instructions: &instructions,
        });

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

            // WASM 1.0 initializer expressions can only contain a single const instruction
            let init = global.init_expr.get_operators_reader().read()?;

            self.prog_builder.handle_global(init);
        }

        Ok(())
    }

    fn handle_tables(&mut self, section: TableSectionReader) -> Result<()> {
        for table in section {
            let table = table?;

            self.prog_builder.handle_table(table.ty.initial as u32);
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

                self.prog_builder.handle_elements(offset, &functions);
            }

            // Other element kinds do not exist in WASM 1.0
        }

        Ok(())
    }
}

//! WALC code generator

mod core;
pub use core::*;

mod function;

mod util;

use crate::analyzer::*;

#[derive(Default)]
pub struct ProgramBuilder {
    util: util::UtilGenerator,

    functions: Vec<code::Code>,

    globals: Vec<number::Number>,

    data_segments: Vec<list::List>,
    data_memory_offsets: Vec<u32>,

    main_id: Option<FuncId>,
    start_id: Option<FuncId>,

    custom_func_table: Vec<Option<FuncId>>,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(mut self) -> Expr {
        let start_function = function::entrypoint(
            &mut self.util,
            &function::EntrypointInfo {
                main_id: self.main_id.unwrap(), // The analyzer must ensure that `main` exists
                start_id: self.start_id,
                data_memory_offsets: &self.data_memory_offsets,
            },
        );

        let start = instruction::start(
            start_function,
            var("FunctionTable"),
            var("CustomTable"),
            var("Globals"),
        );

        let func_count = self.functions.len();

        let mut defs = LetExprBuilder::new();

        generate_core_definitions(&mut defs);

        self.util.generate(&mut defs);

        for (id, data) in self.data_segments.into_iter().enumerate() {
            defs.let_var(format!("Data{id:x}"), data);
        }

        for (id, func) in self.functions.into_iter().enumerate() {
            defs.let_var(format!("Func{id:x}"), func);
        }

        defs.let_var(
            "FunctionTable",
            table::from((0..func_count).map(|id| var(format!("Func{id:x}")))),
        );

        defs.let_var(
            "CustomTable",
            table::from(self.custom_func_table.into_iter().map(|opt_id| {
                if let Some(id) = opt_id {
                    var(format!("Func{id:x}"))
                } else {
                    unreachable()
                }
            })),
        );

        defs.let_var("Globals", table::from(self.globals));

        defs.build_in(start)
    }

    pub fn handle_main(&mut self, id: FuncId) {
        self.main_id = Some(id);
    }

    pub fn handle_start(&mut self, id: FuncId) {
        self.start_id = Some(id);
    }

    pub fn handle_data(&mut self, data: &[u8], target_memory_offset: u32) {
        let list = list::from(data.iter().map(|b| self.util.num.byte_const(*b)));

        self.data_segments.push(list);
        self.data_memory_offsets.push(target_memory_offset);
    }

    pub fn handle_import(&mut self, name: &str) {
        let func = match name {
            "input" => function::input_function(&mut self.util),
            "output" => function::output_function(&mut self.util),
            "exit" => function::exit_function(&mut self.util),
            _ => unreachable!(),
        };

        self.functions.push(func);
    }

    pub fn handle_function(&mut self, func: &Func, types: &GlobalTypeInfo) {
        self.functions
            .push(function::function(&mut self.util, func, types));
    }

    pub fn handle_global(&mut self, init: Operator) {
        self.globals.push(self.util.num.with_init_value(&init));
    }

    pub fn handle_table(&mut self, size: u32) {
        self.custom_func_table.resize(size as usize, None);
    }

    pub fn handle_elements(&mut self, offset: u32, functions: &[FuncId]) {
        for (i, func_id) in functions.iter().enumerate() {
            self.custom_func_table[offset as usize + i] = Some(*func_id);
        }
    }
}

//! WALC code generator

mod core;
pub use core::*;

mod const_expr;
mod function;
mod runtime;

use crate::analyzer::*;

#[derive(Default)]
pub struct ProgramBuilder<'a> {
    runtime: runtime::RuntimeGenerator,

    functions: Vec<code::Code>,

    /// Initializers are stored as explicit instructions and not as generated constants because
    /// they may be used to compute data segment offsets
    globals: Vec<Operator<'a>>,

    memory_initial_size: u32,

    data_segments: Vec<list::List>,
    data_memory_offsets: Vec<number::I32>,

    main_id: Option<FuncId>,
    start_id: Option<FuncId>,

    custom_func_table: Vec<Option<FuncId>>,
}

impl<'a> ProgramBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(mut self) -> Expr {
        let start_function = function::entrypoint(
            &mut self.runtime,
            self.main_id.unwrap(), // The analyzer must ensure that `main` exists
            self.start_id,
            self.memory_initial_size,
            self.data_memory_offsets.into_iter(),
        );

        let start = instruction::start(
            start_function,
            var("FunctionTable"),
            var("CustomTable"),
            var("Globals"),
        );

        let function_table =
            table::from((0..self.functions.len()).map(|id| var(format!("Func{id:x}"))));

        let custom_table = table::from(self.custom_func_table.into_iter().map(|opt_id| {
            if let Some(id) = opt_id {
                var(format!("Func{id:x}"))
            } else {
                unreachable()
            }
        }));

        let globals = table::from(
            self.globals
                .iter()
                .map(|global| self.runtime.num.with_init_value(global)),
        );

        let mut defs = LetExprBuilder::new();

        generate_core_definitions(&mut defs);

        self.runtime.generate(&mut defs);

        for (id, data) in self.data_segments.into_iter().enumerate() {
            defs.def(format!("Data{id:x}"), data);
        }

        for (id, func) in self.functions.into_iter().enumerate() {
            defs.def(format!("Func{id:x}"), func);
        }

        defs.def("FunctionTable", function_table);
        defs.def("CustomTable", custom_table);
        defs.def("Globals", globals);

        defs.build_in(start)
    }

    pub fn handle_main(&mut self, id: FuncId) {
        self.main_id = Some(id);
    }

    pub fn handle_start(&mut self, id: FuncId) {
        self.start_id = Some(id);
    }

    pub fn handle_memory(&mut self, initial_size: u32) {
        self.memory_initial_size = initial_size;
    }

    pub fn handle_global(&mut self, init_expr: &[Operator<'a>]) {
        self.globals.push(const_expr::eval(init_expr, &[]));
    }

    pub fn handle_data(&mut self, data: &[u8], target_memory_offset_expr: &[Operator]) {
        let data = list::from(data.iter().map(|b| self.runtime.num.byte_const(*b)));
        self.data_segments.push(data);

        let offset = const_expr::eval(target_memory_offset_expr, &self.globals);
        let offset = self.runtime.num.with_init_value(&offset);
        self.data_memory_offsets.push(offset);
    }

    pub fn handle_import(&mut self, name: &str) {
        let func = match name {
            "input" => function::input_function(&mut self.runtime),
            "output" => function::output_function(&mut self.runtime),
            "exit" => function::exit_function(&mut self.runtime),
            _ => unreachable!(),
        };

        self.functions.push(func);
    }

    pub fn handle_function(&mut self, func: &Func, types: &GlobalTypeInfo) {
        self.functions
            .push(function::function(&mut self.runtime, func, types));
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

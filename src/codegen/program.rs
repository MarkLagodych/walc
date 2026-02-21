mod function;

use crate::{analyzer::*, codegen::*};

#[derive(Default)]
pub struct ProgramBuilder {
    consts: number::ConstantDefinitionBuilder,
    instrs: function::InstructionDefinitionBuilder,

    functions: Vec<function::InstructionChain>,

    globals: Vec<number::Number>,

    data_segments: Vec<list::List>,
    data_memory_offsets: Vec<u32>,

    main_id: Option<FuncId>,
    start_id: Option<FuncId>,

    custom_func_table: Vec<Option<FuncId>>,
}

fn data_segment(id: usize) -> list::List {
    var(format!("Data{id:x}"))
}

fn function_table() -> table::Table {
    var("FunctionTable")
}

fn custom_table() -> table::Table {
    var("CustomTable")
}

fn globals() -> table::Table {
    var("Globals")
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(mut self) -> Expr {
        let entrypoint = function::entrypoint(
            &function::EntrypointInfo {
                // The analyzer must ensure that a main function exists
                main_id: self.main_id.unwrap(),
                start_id: self.start_id,
                data_memory_offsets: &self.data_memory_offsets,
            },
            &mut self.consts,
            &mut self.instrs,
        );

        let instrs = self.instrs.build(&mut self.consts);
        let consts = self.consts.build();

        let func_count = self.functions.len();

        let mut toplevel = DefinitionBuilder::prelude();
        toplevel.append(consts);
        toplevel.append(instrs);

        for (id, data) in self.data_segments.into_iter().enumerate() {
            toplevel.def(format!("Data{id:x}"), data);
        }

        for (id, func) in self.functions.into_iter().enumerate() {
            toplevel.def(format!("Func{id:x}"), func);
        }

        toplevel.def(
            "FunctionTable",
            table::from((0..func_count).map(|id| var(format!("Func{id:x}")))),
        );

        toplevel.def(
            "CustomTable",
            table::from(self.custom_func_table.into_iter().map(|opt_id| {
                if let Some(id) = opt_id {
                    var(format!("Func{id:x}"))
                } else {
                    unreachable()
                }
            })),
        );

        toplevel.def("Globals", table::from(self.globals));

        toplevel.build(entrypoint)
    }

    pub fn handle_main(&mut self, id: FuncId) {
        self.main_id = Some(id);
    }

    pub fn handle_start(&mut self, id: FuncId) {
        self.start_id = Some(id);
    }

    pub fn handle_data(&mut self, data: &[u8], target_memory_offset: u32) {
        self.data_segments
            .push(list::from_bytes(&mut self.consts, data));

        self.data_memory_offsets.push(target_memory_offset);
    }

    pub fn handle_import(&mut self, name: &str) {
        let func = match name {
            "input" => function::handle_input_function(&mut self.instrs),
            "output" => function::handle_output_function(&mut self.instrs),
            "exit" => function::handle_exit_function(&mut self.instrs),
            _ => unreachable!(),
        };

        self.functions.push(func);
    }

    pub fn handle_function(&mut self, func: &Func, types: &GlobalTypeInfo) {
        self.functions.push(function::handle_function(
            func,
            types,
            &mut self.consts,
            &mut self.instrs,
        ));
    }

    pub fn handle_global(&mut self, init: Operator) {
        self.globals.push(self.consts.with_init_value(&init));
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

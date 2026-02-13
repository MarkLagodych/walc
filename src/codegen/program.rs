use super::*;

use crate::analyzer::*;

#[derive(Default)]
pub struct ProgramBuilder {
    consts: number::ConstantDefinitionBuilder,
    instrs: instruction::InstructionDefinitionBuilder,

    globals: Vec<number::Number>,

    data_segments: Vec<list::List>,
    data_infos: Vec<DataSegmentInfo>,

    functions: Vec<function::FunctionBody>,
    main_id: Option<FuncId>,
    start_id: Option<FuncId>,

    custom_func_table: Vec<Option<u32>>,
}

struct DataSegmentInfo {
    id: DataSegmentId,
    target_memory_offset: u32,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(mut self) -> Expr {
        // The analyzer must ensure that a main function exists
        let main_id = self.main_id.unwrap();
        // TODO main

        if let Some(start_id) = self.start_id {
            // TODO start
        }

        // TODO active data segments

        // TODO root expression
        let expr = io_command::exit();

        let func_count = self.functions.len();

        let mut instr_defs = DefinitionBuilder::new();
        self.instrs.build(&mut instr_defs, &mut self.consts);
        let expr = instr_defs.build(expr);

        let mut toplevel = DefinitionBuilder::prelude();
        self.consts.build(&mut toplevel);

        for (id, data) in self.data_segments.into_iter().enumerate() {
            toplevel.def(format!("Data{id}"), data);
        }

        for (id, func) in self.functions.into_iter().enumerate() {
            toplevel.def(format!("F{id}"), func);
        }

        toplevel.def(
            "FunctionTable",
            table::from((0..func_count).map(|id| var(format!("F{id}")))),
        );

        toplevel.def(
            "CustomTable",
            table::from(
                self.custom_func_table
                    .into_iter()
                    .map(|opt_id| match opt_id {
                        Some(id) => var(format!("F{id}")),
                        None => unreachable(),
                    }),
            ),
        );

        toplevel.def("GlobalTable", table::from(self.globals));

        toplevel.build(expr)
    }

    pub fn handle_main(&mut self, id: FuncId) {
        self.main_id = Some(id);
    }

    pub fn handle_start(&mut self, id: FuncId) {
        self.start_id = Some(id);
    }

    pub fn handle_data(&mut self, id: DataSegmentId, data: &[u8], active_offset: u32) {
        self.data_segments
            .push(list::from_bytes(&mut self.consts, data));

        self.data_infos.push(DataSegmentInfo {
            id,
            target_memory_offset: active_offset,
        });
    }

    pub fn handle_import(&mut self, name: &str) {
        match name {
            "input" => {
                self.functions.push(function::input_function());
            }
            "output" => {
                self.functions.push(function::output_function());
            }
            "exit" => {
                self.functions.push(function::exit_function());
            }
            _ => {}
        }
    }

    pub fn handle_function(&mut self, info: &FunctionInfo) {
        self.functions
            .push(function::function(info, &mut self.consts, &mut self.instrs));
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

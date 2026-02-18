use crate::{analyzer::*, codegen::*};

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
        let expr = unreachable();

        let instr = self.instrs.exit();
        let expr = apply(instr, [expr]);

        // TODO active data segments

        if let Some(start_id) = self.start_id {
            // TODO start
        }

        // The analyzer must ensure that a main function exists
        let main_id = self.main_id.unwrap();
        let instr = self.instrs.call(self.consts.id_const(main_id as u16));
        let expr = apply(instr, [expr]);

        let func_count = self.functions.len();

        let instrs = self.instrs.build(&mut self.consts);
        let consts = self.consts.build();

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
            "IndirectTable",
            table::from(
                self.custom_func_table
                    .into_iter()
                    .map(|opt_id| match opt_id {
                        Some(id) => var(format!("Func{id:x}")),
                        None => unreachable(),
                    }),
            ),
        );

        toplevel.def("Globals", table::from(self.globals));

        toplevel.def("Memory", memory::new());

        let expr = apply(
            expr,
            [
                pair::new(var("FunctionTable"), var("IndirectTable")),
                var("Memory"),
                var("Globals"),
                stack::empty(),
                stack::empty(),
                stack::empty(),
            ],
        );

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
                self.functions
                    .push(function::input_function(&mut self.instrs));
            }
            "output" => {
                self.functions
                    .push(function::output_function(&mut self.instrs));
            }
            "exit" => {
                self.functions
                    .push(function::exit_function(&mut self.instrs));
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

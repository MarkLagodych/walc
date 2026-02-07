use super::*;

use crate::analyzer::*;

/// Global variable initialization expression
type GlobalInitExpr = Expr;
/// A variable that refers to a function
type FuncVar = Expr;

#[derive(Default)]
pub struct ProgramBuilder {
    consts: number::ConstantDefinitionBuilder,

    globals: Vec<GlobalInitExpr>,

    data_segments: DefinitionBuilder,
    active_data_segment_infos: Vec<ActiveDataSegmentInfo>,

    functions: DefinitionBuilder,
    main_id: Option<FuncId>,
    start_id: Option<FuncId>,

    custom_func_table: Vec<FuncVar>,
}

struct ActiveDataSegmentInfo {
    segment_id: DataId,
    offset: u32,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Expr {
        // The analyzer must ensure that a main function exists
        let main_id = self.main_id.unwrap();
        // TODO main

        if let Some(start_id) = self.start_id {
            // TODO start
        }

        // TODO active data segments

        // TODO root expression
        let expr = walc_io::end();

        let mut defs = DefinitionBuilder::new();

        defs.def(
            "FunctionTable",
            table::from((0..self.functions.count()).map(|i| var(format!("F{i}")))),
        );

        defs.def("CustomTable", table::from(self.custom_func_table));

        defs.def("GlobalTable", table::from(self.globals));

        let mut toplevel = DefinitionBuilder::prelude();
        self.consts.build(&mut toplevel);
        self.data_segments.move_to(&mut toplevel);
        self.functions.move_to(&mut toplevel);
        defs.move_to(&mut toplevel);
        toplevel.build(expr)
    }

    pub fn handle_import(&mut self, name: &str, id: FuncId) {
        match name {
            "input" => {
                self.functions
                    .def(format!("F{id}"), function::input_function());
            }
            "output" => {
                self.functions
                    .def(format!("F{id}"), function::output_function());
            }
            "exit" => {
                self.functions
                    .def(format!("F{id}"), function::exit_function());
            }
            _ => {}
        }
    }

    pub fn handle_main(&mut self, id: FuncId) {
        self.main_id = Some(id);
    }

    pub fn handle_start(&mut self, id: FuncId) {
        self.start_id = Some(id);
    }

    pub fn handle_data(&mut self, id: DataId, data: &[u8], active_offset: Option<u32>) {
        self.data_segments
            .def(format!("DAT{id}"), list::from_bytes(&mut self.consts, data));

        if let Some(offset) = active_offset {
            self.active_data_segment_infos.push(ActiveDataSegmentInfo {
                segment_id: id,
                offset,
            });
        }
    }

    pub fn handle_function(
        &mut self,
        id: FuncId,
        param_count: u32,
        has_result: bool,
        local_types: &[ValType],
        operators: &[Operator],
    ) {
        self.functions.def(
            format!("F{id}"),
            function::function(param_count, has_result, local_types, operators),
        );
    }

    pub fn handle_global(&mut self, ty: ValType, init: u64) {
        let expr = match ty {
            ValType::I32 => self.consts.i32_const(init as u32),
            ValType::I64 => self.consts.i64_const(init),
            ValType::F32 => self.consts.i32_const(init as u32),
            ValType::F64 => self.consts.i64_const(init),
            _ => unreachable!(),
        };

        self.globals.push(expr);
    }

    pub fn handle_table(&mut self, size: u32) {
        self.custom_func_table.resize(size as usize, unreachable());
    }

    pub fn handle_elements(&mut self, offset: u32, functions: &[FuncId]) {
        for (i, func_id) in functions.iter().enumerate() {
            self.custom_func_table[offset as usize + i] = var(format!("F{func_id}"));
        }
    }
}

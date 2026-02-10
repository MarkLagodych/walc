use super::*;

use crate::analyzer::*;

/// Global variable initialization expression
type GlobalInitExpr = Expr;
// TODO this definition is too specific. Create number::from and number::default that handle
// ValType and ValType+value

#[derive(Default)]
pub struct ProgramBuilder {
    pub consts: number::ConstantDefinitionBuilder,

    globals: Vec<GlobalInitExpr>,

    data_segments: Vec<list::List>,
    active_data_segment_infos: Vec<ActiveDataSegmentInfo>,

    functions: Vec<function::FunctionBody>,
    main_id: Option<FuncId>,
    start_id: Option<FuncId>,

    custom_func_table: Vec<Option<u32>>,
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

        let func_count = self.functions.len();

        let mut defs = DefinitionBuilder::prelude();

        self.consts.build(&mut defs);

        for (i, data) in self.data_segments.into_iter().enumerate() {
            defs.def(format!("Data{i}"), data);
        }

        for (i, func) in self.functions.into_iter().enumerate() {
            defs.def(format!("F{i}"), func);
        }

        defs.def(
            "FunctionTable",
            table::from((0..func_count).map(|i| var(format!("F{i}")))),
        );

        defs.def(
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

        defs.def("GlobalTable", table::from(self.globals));

        defs.build(expr)
    }

    pub fn handle_main(&mut self, id: FuncId) {
        self.main_id = Some(id);
    }

    pub fn handle_start(&mut self, id: FuncId) {
        self.start_id = Some(id);
    }

    pub fn handle_data(&mut self, id: DataId, data: &[u8], active_offset: Option<u32>) {
        self.data_segments
            .push(list::from_bytes(&mut self.consts, data));

        if let Some(offset) = active_offset {
            self.active_data_segment_infos.push(ActiveDataSegmentInfo {
                segment_id: id,
                offset,
            });
        }
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

    pub fn handle_function(
        &mut self,
        param_count: u32,
        has_result: bool,
        local_types: &[ValType],
        instructions: &[Operator],
    ) {
        self.functions.push(function::function(
            param_count,
            has_result,
            local_types,
            instructions,
        ));
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
        self.custom_func_table.resize(size as usize, None);
    }

    pub fn handle_elements(&mut self, offset: u32, functions: &[FuncId]) {
        for (i, func_id) in functions.iter().enumerate() {
            self.custom_func_table[offset as usize + i] = Some(*func_id);
        }
    }
}

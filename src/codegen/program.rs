use super::*;

use crate::analyzer::*;

#[derive(Default)]
pub struct ProgramBuilder {
    consts: number::ConstantDefinitionBuilder,

    globals: Vec<Expr>,

    data_segments: DefinitionBuilder,
    active_data_segment_infos: Vec<ActiveDataSegmentInfo>,

    functions: DefinitionBuilder,
    main_id: Option<FuncId>,
    walc_input_id: Option<FuncId>,
    walc_output_id: Option<FuncId>,
    walc_exit_id: Option<FuncId>,
    start_id: Option<FuncId>,

    custom_func_table: Vec<Expr>,
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

        if let Some(start_id) = self.start_id {
            // TODO
        }

        // TODO handle active data segments
        let mut expr = walc_io::end();

        let mut defs = DefinitionBuilder::new();

        defs.def(
            "GlobalTable",
            list::from((0..self.functions.count()).map(|i| var(format!("F{i}")))),
        );

        defs.def(
            "CustomTable",
            list::from(self.custom_func_table.into_iter()),
        );

        defs.def("G", table::from(self.globals));

        expr = defs.build(expr);
        expr = self.functions.build(expr);
        expr = self.data_segments.build(expr);

        let mut toplevel = DefinitionBuilder::prelude();
        self.consts.build(&mut toplevel);
        toplevel.build(expr)
    }

    pub fn handle_import(&mut self, name: &str, id: FuncId) {
        match name {
            "input" => self.walc_input_id = Some(id),
            "output" => self.walc_output_id = Some(id),
            "exit" => self.walc_exit_id = Some(id),
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

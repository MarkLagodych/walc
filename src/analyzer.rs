use crate::{codegen, parser::*};

use anyhow::{Result, anyhow};

#[derive(Default)]
pub struct Analyzer {
    defs: codegen::DefinitionBuilder,
    consts: codegen::number::ConstantSet,

    function_info: FunctionInfo,
    active_data_segment_infos: Vec<ActiveDataSegmentInfo>,
}

#[derive(Default)]
struct FunctionInfo {
    main_id: Option<FuncId>,
    walc_input_id: Option<FuncId>,
    walc_output_id: Option<FuncId>,
    walc_exit_id: Option<FuncId>,
    start_id: Option<FuncId>,
}

struct ActiveDataSegmentInfo {
    data_segment_id: u32,
    offset_expr: codegen::Expr,
}

impl Analyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compile(self) -> codegen::Expr {
        // The parser must ensure that a main function exists
        let main_id = self.function_info.main_id.unwrap();

        if let Some(start_id) = self.function_info.start_id {
            // TODO
        }

        // TODO handle active data segments

        // TODO
        let root_expr = codegen::walc_io::end();

        let mut toplevel = codegen::DefinitionBuilder::new();
        toplevel.define_prelude();
        self.consts.define_constants(&mut toplevel);

        let mut expr = self.defs.build(root_expr);
        expr = toplevel.build(expr);

        expr
    }

    pub fn handle_import(&mut self, name: &str, id: FuncId) {}

    pub fn handle_main(&mut self, id: FuncId) {}

    pub fn handle_start(&mut self, id: FuncId) {}

    pub fn handle_data(&mut self, id: DataId, data: &[u8], active_offset: Option<u32>) {}

    pub fn handle_function(
        &mut self,
        id: FuncId,
        param_count: u32,
        has_result: bool,
        local_types: &[ValType],
        operators: &[Operator],
    ) {
    }

    pub fn handle_global(&mut self, id: GlobalId, ty: ValType, init: u64) {}

    pub fn handle_table(&mut self, size: u32) {}

    pub fn handle_elements(&mut self, offset: u32, functions: &[FuncId]) {}
}

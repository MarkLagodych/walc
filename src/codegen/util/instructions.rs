mod control_flow;
mod memory;
mod numeric;
mod variable;
mod walc_io;

use super::*;

use crate::codegen::{
    function::BlockStack,
    instruction::{Instruction, InstructionBuilder},
};

use crate::analyzer::{Func, Operator};

impl UtilGenerator {
    pub fn instruction(&mut self, op: &Operator, blocks: &BlockStack) -> Instruction {
        use Operator::*;

        match op {
            // Parametric instructions ///////////////////////////////////////
            Drop => self.drop(),
            Select => self.select(),

            // Constrol flow instructions /////////////////////////////////////
            Nop => instruction::nop(),
            Unreachable => self.exit(),

            Call { function_index } => self.call(*function_index),
            CallIndirect { .. } => self.call_indirect(),

            Loop { .. } | If { .. } | Block { .. } => self.begin_block(blocks),
            Else => self.block_else(blocks),
            End => self.end_block(blocks),

            Br { relative_depth } => self.br(blocks, *relative_depth),
            BrIf { relative_depth } => self.br_if(blocks, *relative_depth),
            BrTable { targets } => self.br_table(blocks, targets),
            Return => self.ret(blocks),

            // Variable instructions ///////////////////////////////////////////
            LocalGet { local_index } => self.local_get(*local_index),
            LocalSet { local_index } => self.local_set(*local_index),
            LocalTee { local_index } => self.local_tee(*local_index),
            GlobalGet { global_index } => self.global_get(*global_index),
            GlobalSet { global_index } => self.global_set(*global_index),

            // Memory instructions //////////////////////////////////////////
            MemorySize { .. } => self.memory_size(),
            MemoryGrow { .. } => self.memory_grow(),
            I32Load { memarg, .. } => self.i32_load(memarg.offset as u32),
            I64Load { memarg, .. } => self.i64_load(memarg.offset as u32),

            // Numeric instructions //////////////////////////////////////////
            I32Const { .. } | I64Const { .. } | F32Const { .. } | F64Const { .. } => {
                self.const_push(op)
            }

            I32WrapI64 => self.i32_wrap_i64(),

            I32Eqz | I64Eqz => self.eqz(),
            I32Eq | I64Eq => self.eq(),
            I32Ne | I64Ne => self.ne(),

            I32And | I64And => self.and(),
            I32Or | I64Or => self.or(),
            I32Xor | I64Xor => self.xor(),

            I32LtU | I64LtU => self.lt_u(),
            I32LeU | I64LeU => self.le_u(),
            I32GtU | I64GtU => self.gt_u(),
            I32GeU | I64GeU => self.ge_u(),

            I32LtS | I64LtS => self.lt_s(),
            I32LeS | I64LeS => self.le_s(),
            I32GtS | I64GtS => self.gt_s(),
            I32GeS | I64GeS => self.ge_s(),

            I32Add | I64Add => self.add(),
            I32Sub | I64Sub => self.sub(),

            // TODO
            _ => todo!(),
        }
    }

    pub fn func_prologue(&mut self, func: &Func) -> Instruction {
        let mut b = InstructionBuilder::new();

        let param_count = func.func_type.params().len();

        b.pop((0..param_count).map(|i| format!("p{i:x}")));

        let mut locals = Vec::new();
        locals.extend((0..param_count).map(|i| var(format!("p{i:x}"))));
        locals.extend(
            func.local_types
                .iter()
                .map(|ty| self.num.default_const(*ty)),
        );

        b.push_locals_frame(table::from(locals));
        b.push_stack_frame();

        b.build()
    }

    fn drop(&mut self) -> Instruction {
        if !self.has("Drop") {
            self.def("Drop", {
                let mut b = InstructionBuilder::new();
                b.pop(["a"]);
                b.build()
            });
        }

        var("Drop")
    }

    fn select(&mut self) -> Instruction {
        if !self.has("Select") {
            let definition = {
                let mut b = InstructionBuilder::new();

                b.pop(["a", "b", "c"]);

                b.push([select(self.num_is_zero(var("c")), var("a"), var("b"))]);

                b.build()
            };

            self.def("Select", definition);
        }

        var("Select")
    }
}

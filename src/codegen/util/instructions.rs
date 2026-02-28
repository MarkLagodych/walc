use super::*;

use crate::codegen::{
    function::{BlockLabels, BlockStack},
    instruction::{Instruction, InstructionBuilder},
};

use crate::analyzer::*;

impl UtilGenerator {
    pub fn instruction(&mut self, op: &Operator, blocks: &BlockStack) -> Instruction {
        use Operator::*;

        match op {
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

            I32Const { .. } | I64Const { .. } | F32Const { .. } | F64Const { .. } => {
                self.push_const(op)
            }

            LocalGet { local_index } => self.local_get(*local_index),
            LocalSet { local_index } => self.local_set(*local_index),
            GlobalGet { global_index } => self.global_get(*global_index),
            GlobalSet { global_index } => self.global_set(*global_index),

            // TODO
            _ => todo!(),
        }
    }

    pub fn output_and_return(&mut self) -> Instruction {
        if !self.has("Output") {
            self.def("Output", {
                let mut b = InstructionBuilder::new();
                b.pop(["a"]);
                // TODO convert a to byte
                let byte = number::reverse_bits(var("a"));
                b.ret();
                b.build_output(byte)
            });
        }

        var("Output")
    }

    pub fn input_and_return(&mut self) -> Instruction {
        if !self.has("Input") {
            let invalid_input = self.num.i32_const(u32::MAX);

            self.def("Input", {
                let mut b = InstructionBuilder::new();
                b.push([select(
                    optional::is_some(var("inp")),
                    invalid_input,
                    // TODO convert input to i32
                    optional::unwrap(var("inp")),
                )]);
                b.ret();
                b.build_input("inp")
            });
        }

        var("Input")
    }

    pub fn exit(&mut self) -> Instruction {
        if !self.has("Exit") {
            self.def("Exit", {
                let b = InstructionBuilder::new();
                b.build_exit()
            });
        }

        var("Exit")
    }

    pub fn call(&mut self, function_id: FuncId) -> Instruction {
        if !self.has("Call") {
            self.def("Call", {
                abs(["funcid"], {
                    let mut b = InstructionBuilder::new();
                    b.call(var("funcid"));
                    b.build()
                })
            });
        }

        let id = self.num.id_const(function_id as u16);
        apply(var("Call"), [id])
    }

    pub fn call_indirect(&mut self) -> Instruction {
        if !self.has("CallIndirect") {
            self.def("CallIndirect", {
                let mut b = InstructionBuilder::new();
                b.pop(["a"]);
                b.call_indirect(var("a"));
                b.build()
            });
        }

        var("CallIndirect")
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

    fn begin_block(&mut self, blocks: &BlockStack) -> Instruction {
        let block = blocks.get(0);

        let mut b = InstructionBuilder::new();

        match &block.labels {
            BlockLabels::Loop => {
                b.push_trace(b.next());
            }
            BlockLabels::If { else_label, .. } => {
                b.pop(["cond"]);

                b.set_next(select(
                    self.num_is_not_zero(var("cond")),
                    else_label.clone(),
                    b.next(),
                ));
            }
            _ => {}
        }

        let param_count = block.block_type.param_count;

        b.pop((0..param_count).map(|i| format!("p{i:x}")));

        b.push_stack_frame();

        b.push((0..param_count).map(|i| var(format!("p{i:x}"))));

        b.build()
    }

    fn end_block(&self, blocks: &BlockStack) -> Instruction {
        let block = blocks.get(0);

        let mut b = InstructionBuilder::new();

        let result_count = block.block_type.result_count;

        b.pop((0..result_count).map(|i| format!("r{i:x}")));

        b.pop_stack_frame();

        b.push((0..result_count).map(|i| var(format!("r{i:x}"))));

        match block.labels {
            BlockLabels::Loop => {
                b.drop_trace();
            }
            BlockLabels::Func { .. } => {
                b.pop_locals_frame();
                b.ret();
            }
            _ => {}
        }

        b.build()
    }

    fn block_else(&mut self, blocks: &BlockStack) -> Instruction {
        let mut b = InstructionBuilder::new();

        match &blocks.get(0).labels {
            BlockLabels::If { end_label, .. } => {
                b.set_next(end_label.clone());
            }
            _ => unreachable!(),
        }

        b.build()
    }

    fn ret(&mut self, blocks: &BlockStack) -> Instruction {
        self.br(blocks, blocks.get_outermost_index())
    }

    fn br(&mut self, blocks: &BlockStack, depth: u32) -> Instruction {
        let mut b = InstructionBuilder::new();

        let target_block = blocks.get(depth);
        let pop_count = match &target_block.labels {
            BlockLabels::Loop => target_block.block_type.param_count,
            _ => target_block.block_type.result_count,
        };

        b.pop((0..pop_count).map(|i| format!("x{i:x}")));

        if depth > 0 {
            for i in 0..depth - 1 {
                b.pop_stack_frame();

                if matches!(blocks.get(i).labels, BlockLabels::Loop) {
                    b.drop_trace();
                }
            }
        }

        b.push((0..pop_count).map(|i| var(format!("x{i:x}"))));

        match &target_block.labels {
            BlockLabels::Loop => {
                b.get_trace_top("loop");
                b.set_next(var("loop"));
            }
            BlockLabels::If { end_label, .. }
            | BlockLabels::Block { end_label }
            | BlockLabels::Func { end_label } => {
                b.set_next(end_label.clone());
            }
        }

        b.build()
    }

    fn br_if(&mut self, blocks: &BlockStack, depth: u32) -> Instruction {
        let mut b = InstructionBuilder::new();

        b.pop(["cond"]);

        b.set_next(select(
            self.num_is_not_zero(var("cond")),
            b.next(),
            self.br(blocks, depth),
        ));

        b.build()
    }

    fn br_table(&mut self, blocks: &BlockStack, targets: &BrTable) -> Instruction {
        let mut b = InstructionBuilder::new();

        b.pop(["idx"]);

        let mut next = self.br(blocks, targets.default());

        // Here `targets` contain parsing `Result`s.
        // The validator should catch all parsing errors, so we can just unwrap it.
        let break_targets = targets.targets().collect::<Result<Vec<_>, _>>().unwrap();

        for (i, target) in break_targets.into_iter().enumerate().rev() {
            let i = self.num.i32_const(i as u32);
            let target = self.br(blocks, target);
            next = select(self.num_equal(var("idx"), i), next, target);
        }

        b.set_next(next);

        b.build()
    }

    fn push_const(&mut self, op: &Operator) -> Instruction {
        if !self.has("Push") {
            self.def("Push", {
                abs(["item"], {
                    let mut b = InstructionBuilder::new();
                    b.push([var("item")]);
                    b.build()
                })
            });
        }

        let item = self.num.with_init_value(op);
        apply(var("Push"), [item])
    }

    fn local_get(&mut self, local_index: u32) -> Instruction {
        if !self.has("LGet") {
            self.def("LGet", {
                abs(["id"], {
                    let mut b = InstructionBuilder::new();
                    b.get_local("a", var("id"));
                    b.push([var("a")]);
                    b.build()
                })
            });
        }

        let id = self.num.id_const(local_index as u16);
        apply(var("LGet"), [id])
    }

    fn local_set(&mut self, local_index: u32) -> Instruction {
        if !self.has("LSet") {
            self.def("LSet", {
                abs(["id"], {
                    let mut b = InstructionBuilder::new();
                    b.pop(["a"]);
                    b.set_local(var("id"), var("a"));
                    b.build()
                })
            });
        }

        let id = self.num.id_const(local_index as u16);
        apply(var("LSet"), [id])
    }

    // TODO local.tee

    fn global_get(&mut self, global_index: u32) -> Instruction {
        if !self.has("GGet") {
            self.def("GGet", {
                abs(["id"], {
                    let mut b = InstructionBuilder::new();
                    b.get_global("a", var("id"));
                    b.push([var("a")]);
                    b.build()
                })
            });
        }

        let id = self.num.id_const(global_index as u16);
        apply(var("GGet"), [id])
    }

    fn global_set(&mut self, global_index: u32) -> Instruction {
        if !self.has("GSet") {
            self.def("GSet", {
                abs(["id"], {
                    let mut b = InstructionBuilder::new();
                    b.pop(["a"]);
                    b.set_global(var("id"), var("a"));
                    b.build()
                })
            });
        }

        let id = self.num.id_const(global_index as u16);
        apply(var("GSet"), [id])
    }
}

use super::*;

use function::*;
use instruction::{Instruction, InstructionBuilder};

use crate::analyzer::*;

pub struct InstructionInfo<'a> {
    pub op: &'a Operator<'a>,
    pub types: &'a GlobalTypeInfo,
    pub blocks: &'a BlockStack,
}

impl UtilGenerator {
    pub fn instruction(&mut self, info: &mut InstructionInfo) -> Instruction {
        use Operator::*;

        match info.op {
            I32Const { .. } | I64Const { .. } | F32Const { .. } | F64Const { .. } => {
                let num = self.num.with_init_value(info.op);
                self.push(num)
            }

            Call { function_index } => self.call(*function_index),

            CallIndirect { .. } => self.call_indirect(),

            // TODO else, br, be_if, etc.
            Loop { .. } | If { .. } | Block { .. } => self.enter_block(info.blocks),

            End => self.leave_block(info.blocks),

            LocalGet { local_index } => self.local_get(*local_index),
            LocalSet { local_index } => self.local_set(*local_index),
            GlobalGet { global_index } => self.global_get(*global_index),
            GlobalSet { global_index } => self.global_set(*global_index),

            Return => {
                // TODO jump to the end
                self.nop()
            }

            // TODO
            _ => todo!(),
        }
    }

    fn nop(&mut self) -> Instruction {
        if !self.has("Nop") {
            self.def("Nop", {
                let b = InstructionBuilder::new();
                b.build()
            });
        }
        var("Nop")
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

    pub fn push(&mut self, item: Expr) -> Instruction {
        if !self.has("Push") {
            self.def("Push", {
                abs(["item"], {
                    let mut b = InstructionBuilder::new();
                    b.push([var("item")]);
                    b.build()
                })
            });
        }

        apply(var("Push"), [item])
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

    pub fn enter(&mut self, func: &Func) -> Instruction {
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

    pub fn leave(&self, func_type: &FuncType) -> Instruction {
        let mut b = InstructionBuilder::new();

        let result_count = func_type.results().len();

        b.pop((0..result_count).map(|i| format!("r{i:x}")));

        b.pop_locals_frame();
        b.pop_stack_frame();

        b.push((0..result_count).map(|i| var(format!("r{i:x}"))));

        b.ret();

        b.build()
    }

    fn enter_block(&mut self, blocks: &BlockStack) -> Instruction {
        let block = blocks.get(0);
        let param_count = block.type_info.param_count;

        let mut b = InstructionBuilder::new();

        match &block.label_info {
            BlockLabelInfo::Loop => {
                b.push_trace(b.next());
            }
            BlockLabelInfo::If { else_label, .. } => {
                b.pop(["cond"]);

                b.set_next(select(
                    self.num_is_not_zero(var("cond")),
                    else_label.clone(),
                    b.next(),
                ));
            }
            _ => {}
        }

        b.pop((0..param_count).map(|i| format!("p{i:x}")));

        b.push_stack_frame();

        b.push((0..param_count).map(|i| var(format!("p{i:x}"))));

        b.build()
    }

    fn leave_block(&self, blocks: &BlockStack) -> Instruction {
        let block = blocks.get(0);
        let result_count = block.type_info.result_count;

        let mut b = InstructionBuilder::new();

        b.pop((0..result_count).map(|i| format!("r{i:x}")));

        b.pop_stack_frame();

        b.push((0..result_count).map(|i| var(format!("r{i:x}"))));

        if let BlockLabelInfo::Loop = block.label_info {
            b.drop_trace();
        }

        b.build()
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

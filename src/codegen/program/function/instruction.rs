mod builder;
use builder::*;

mod arith;
use arith::*;

use crate::{analyzer::*, codegen::*};

use super::labels::*;

use std::collections::BTreeMap as Map;

/// An instruction is a function of `(N, F, M, G, L, S, T) -> IoCommand` where:
/// - `N` is the next instruction (or `unreachable` is this is the last instruction in a function)
/// - `F` is a pair of the global function table and the user-defined table for indirect calls
/// - `M` is the memory
/// - `G` is the global variable table
/// - `L` is the local variable table
/// - `S` is the data stack
/// - `T` is the trace (i.e. control flow stack)
///
/// See also [`io_command::IoCommand`].
///
/// An instruction might return an `IoCommand` directly or by invoking the next instruction
/// (or any other instruction in general).
pub type Instruction = Expr;

pub fn start(entrypoint: Instruction) -> io_command::IoCommand {
    apply(
        entrypoint,
        [
            pair::new(program::function_table(), program::custom_table()),
            memory::new(),
            program::globals(),
            locals::new(),
            data_stack::empty(),
            stack::empty(),
        ],
    )
}

pub struct InstructionBuildInfo<'a> {
    pub op: &'a Operator<'a>,
    pub types: &'a GlobalTypeInfo,
    pub consts: &'a mut number::ConstantDefinitionBuilder,
    pub labels: LabelInfo<'a>,
}

#[derive(Default)]
pub struct InstructionDefinitionBuilder {
    map: Map<String, InstructionDefinitionFn>,
    arith: ArithDefinitionBuilder,
}

struct InstructionDefinitionContext<'a> {
    consts: &'a mut number::ConstantDefinitionBuilder,
    arith: &'a mut ArithDefinitionBuilder,
}

type InstructionDefinitionFn = fn(&mut InstructionDefinitionContext) -> Expr;

impl InstructionDefinitionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an instruction definition with the given name and definition function.
    fn def(&mut self, name: impl ToString, def: InstructionDefinitionFn) {
        self.map.insert(name.to_string(), def);
    }

    pub fn build(mut self, consts: &mut number::ConstantDefinitionBuilder) -> DefinitionBuilder {
        let mut instr_defs = DefinitionBuilder::new();

        let mut ctx = InstructionDefinitionContext {
            consts,
            arith: &mut self.arith,
        };

        for (def_name, def) in self.map.into_iter() {
            instr_defs.def(def_name, def(&mut ctx));
        }

        let mut b = self.arith.build(consts);
        b.append(instr_defs);
        b
    }

    pub fn instruction(&mut self, info: &mut InstructionBuildInfo) -> Instruction {
        use Operator::*;

        match info.op {
            I32Const { .. } | I64Const { .. } | F32Const { .. } | F64Const { .. } => {
                self.push(info.consts.with_init_value(info.op))
            }

            Call { function_index } => self.call(info.consts.id_const(*function_index as u16)),
            CallIndirect { .. } => self.call_indirect(),

            Loop { .. } | If { .. } | Block { .. } => {
                self.enter_block(info.op, info.types, &info.labels)
            }

            End => self.leave_block(info.types, &info.labels),

            LocalGet { local_index } => self.local_get(info.consts.id_const(*local_index as u16)),
            LocalSet { local_index } => self.local_set(info.consts.id_const(*local_index as u16)),
            GlobalGet { global_index } => {
                self.global_get(info.consts.id_const(*global_index as u16))
            }
            GlobalSet { global_index } => {
                self.global_set(info.consts.id_const(*global_index as u16))
            }

            Return => {
                // TODO jump to the end
                self.nop()
            }

            // TODO
            _ => todo!(),
        }
    }

    fn nop(&mut self) -> Instruction {
        self.def("Nop", |_| {
            let b = InstructionBuilder::new();
            b.build()
        });

        var("Nop")
    }

    pub fn output_and_return(&mut self) -> Instruction {
        self.def("Output", |_| {
            let mut b = InstructionBuilder::new();
            b.pop(["a"]);
            // TODO convert a to byte
            b.set_output(number::reverse_bits(var("a")));
            b.ret();
            b.build()
        });

        var("Output")
    }

    pub fn input_and_return(&mut self) -> Instruction {
        self.def("Input", |ctx| {
            let mut b = InstructionBuilder::new();
            b.set_input("inp");
            b.push([select(
                optional::is_some(var("inp")),
                ctx.consts.i32_const(u32::MAX),
                // TODO convert input to i32
                optional::unwrap(var("inp")),
            )]);
            b.ret();
            b.build()
        });

        var("Input")
    }

    pub fn exit(&mut self) -> Instruction {
        self.def("Exit", |_| {
            let mut b = InstructionBuilder::new();
            b.set_exit();
            b.build()
        });

        var("Exit")
    }

    pub fn push(&mut self, item: Expr) -> Instruction {
        self.def("Push", |_| {
            abs(["item"], {
                let mut b = InstructionBuilder::new();
                b.push([var("item")]);
                b.build()
            })
        });

        apply(var("Push"), [item])
    }

    pub fn call(&mut self, function_id: number::Id) -> Instruction {
        self.def("Call", |_| {
            abs(["funcid"], {
                let mut b = InstructionBuilder::new();
                b.call(var("funcid"));
                b.build()
            })
        });

        apply(var("Call"), [function_id])
    }

    pub fn call_indirect(&mut self) -> Instruction {
        self.def("CallIndirect", |_| {
            let mut b = InstructionBuilder::new();
            b.pop(["a"]);
            b.call_indirect(var("a"));
            b.build()
        });

        var("CallIndirect")
    }

    pub fn enter(
        &self,
        func: &Func,
        consts: &mut number::ConstantDefinitionBuilder,
    ) -> Instruction {
        let mut b = InstructionBuilder::new();

        let param_count = func.func_type.params().len();

        b.pop((0..param_count).map(|i| format!("p{i:x}")));

        let mut locals = Vec::new();
        locals.extend((0..param_count).map(|i| var(format!("p{i:x}"))));
        locals.extend(func.local_types.iter().map(|ty| consts.default_const(*ty)));

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

    fn enter_block(
        &mut self,
        block: &Operator,
        types: &GlobalTypeInfo,
        labels: &LabelInfo,
    ) -> Instruction {
        let func_type = match block {
            Operator::Loop { blockty } | Operator::If { blockty } | Operator::Block { blockty } => {
                match blockty {
                    BlockType::Empty => &FuncType::new([], []),
                    BlockType::Type(type_id) => &FuncType::new([], [*type_id]),
                    BlockType::FuncType(func_type) => types.get_type(*func_type),
                }
            }
            _ => unreachable!(),
        };

        let param_count = func_type.params().len();

        let mut b = InstructionBuilder::new();

        match block {
            Operator::Loop { .. } => {
                b.push_trace(b.next());
            }
            Operator::If { .. } => {
                let branch0 = match &labels.else_label {
                    Some(label) => label.clone(),
                    None => labels.end_labels.last().unwrap().clone(),
                };

                b.pop(["cond"]);

                b.set_next(select(self.arith.neqz(var("cond")), branch0, b.next()));
            }
            _ => {}
        }

        b.pop((0..param_count).map(|i| format!("p{i:x}")));

        b.push_stack_frame();

        b.push((0..param_count).map(|i| var(format!("p{i:x}"))));

        b.build()
    }

    fn leave_block(&self, types: &GlobalTypeInfo, labels: &LabelInfo) -> Instruction {
        let block = labels.blocks.last().unwrap();

        let func_type = match block {
            Operator::Loop { blockty } | Operator::If { blockty } | Operator::Block { blockty } => {
                match blockty {
                    BlockType::Empty => &FuncType::new([], []),
                    BlockType::Type(type_id) => &FuncType::new([], [*type_id]),
                    BlockType::FuncType(func_type) => types.get_type(*func_type),
                }
            }
            _ => unreachable!(),
        };

        let result_count = func_type.results().len();

        let mut b = InstructionBuilder::new();

        b.pop((0..result_count).map(|i| format!("r{i:x}")));

        b.pop_stack_frame();

        b.push((0..result_count).map(|i| var(format!("r{i:x}"))));

        if let Operator::Loop { .. } = block {
            b.drop_trace();
        }

        b.build()
    }

    fn local_get(&mut self, local_index: number::Id) -> Instruction {
        self.def("LGet", |_| {
            abs(["id"], {
                let mut b = InstructionBuilder::new();
                b.get_local("a", var("id"));
                b.push([var("a")]);
                b.build()
            })
        });

        apply(var("LGet"), [local_index])
    }

    fn local_set(&mut self, local_index: number::Id) -> Instruction {
        self.def("LSet", |_| {
            abs(["id"], {
                let mut b = InstructionBuilder::new();
                b.pop(["a"]);
                b.set_local(var("id"), var("a"));
                b.build()
            })
        });

        apply(var("LSet"), [local_index])
    }

    // TODO local.tee

    fn global_get(&mut self, global_index: number::Id) -> Instruction {
        self.def("GGet", |_| {
            abs(["id"], {
                let mut b = InstructionBuilder::new();
                b.get_global("a", var("id"));
                b.push([var("a")]);
                b.build()
            })
        });

        apply(var("GGet"), [global_index])
    }

    fn global_set(&mut self, global_index: number::Id) -> Instruction {
        self.def("GSet", |_| {
            abs(["id"], {
                let mut b = InstructionBuilder::new();
                b.pop(["a"]);
                b.set_global(var("id"), var("a"));
                b.build()
            })
        });

        apply(var("GSet"), [global_index])
    }
}

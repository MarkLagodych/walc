mod builder;
use builder::*;

mod arith;
use arith::*;

use crate::{analyzer::*, codegen::*};

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

pub struct InstructionBuildInfo<'a> {
    pub op: &'a Operator<'a>,
    pub types: &'a GlobalTypeInfo,
    pub consts: &'a mut number::ConstantDefinitionBuilder,
    pub end_labels: &'a [function::EndLabel<'a>],
    pub else_labels: &'a [function::ElseLabel],
}

struct InstructionDefinitionContext<'a> {
    consts: &'a mut number::ConstantDefinitionBuilder,
}

type InstructionDefinitionFn = fn(&mut InstructionDefinitionContext) -> Expr;

#[derive(Default)]
pub struct InstructionDefinitionBuilder {
    map: Map<String, InstructionDefinitionFn>,
}

impl InstructionDefinitionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an instruction definition with the given name and definition function.
    fn def(&mut self, name: impl ToString, def: InstructionDefinitionFn) {
        self.map.insert(name.to_string(), def);
    }

    pub fn build(self, consts: &mut number::ConstantDefinitionBuilder) -> DefinitionBuilder {
        let mut b = DefinitionBuilder::new();

        // TODO use ArithDefBuilder here
        let mut ctx = InstructionDefinitionContext { consts };

        for (def_name, def) in self.map.into_iter() {
            b.def(def_name, def(&mut ctx));
        }

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

            End => {
                todo!()
            }

            // TODO
            _ => todo!(),
        }
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

        b.push_frame(table::from(locals));

        b.build()
    }

    pub fn leave(&self, func: &Func) -> Instruction {
        let mut b = InstructionBuilder::new();

        let result_count = func.func_type.results().len();

        b.pop((0..result_count).map(|i| format!("r{i:x}")));

        b.pop_frame();

        b.push((0..result_count).map(|i| var(format!("r{i:x}"))));

        b.ret();
        b.build()
    }
}

mod context;
use context::*;

use crate::{analyzer::*, codegen::*};

use std::collections::BTreeMap as Map;

pub type Instruction = Expr;

struct InstructionDefinitionContext<'a> {
    consts: &'a mut number::ConstantDefinitionBuilder,
}

use InstructionDefinitionContext as DefCtx;

#[derive(Default)]
pub struct InstructionDefinitionBuilder {
    map: Map<String, fn(&mut DefCtx) -> Expr>,
}

impl InstructionDefinitionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    fn insert(&mut self, name: impl ToString, def: fn(&mut DefCtx) -> Expr) {
        self.map.insert(name.to_string(), def);
    }

    pub fn num_const(&mut self, num: number::Number) -> Expr {
        self.insert("NumConst", Self::num_const_def);

        apply(var("NumConst"), [num])
    }

    fn num_const_def(_ctx: &mut DefCtx) -> Expr {
        abs(
            ["item"],
            instr({
                ContextBuilder::new()
                    .stack(framed_stack::push(stack(), var("item")))
                    .build()
            }),
        )
    }

    pub fn call(&mut self, function_id: number::Id) -> Expr {
        self.insert("Call", Self::call_def);

        apply(var("Call"), [function_id])
    }

    fn call_def(_ctx: &mut DefCtx) -> Expr {
        abs(
            ["func"],
            instr({
                let func = table::index(functions(), var("func"));

                ContextBuilder::new()
                    .trace(stack::push(trace(), next()))
                    .next(func)
                    .build()
            }),
        )
    }

    pub fn build(self, b: &mut DefinitionBuilder, consts: &mut number::ConstantDefinitionBuilder) {
        // TODO use ArithDefBuilder here

        let mut ctx = DefCtx { consts };

        for (def_name, def) in self.map.into_iter() {
            b.def(def_name, def(&mut ctx));
        }
    }

    pub fn instruction(
        &mut self,
        op: &Operator,
        info: &FunctionInfo,
        consts: &mut number::ConstantDefinitionBuilder,
    ) -> Instruction {
        match op {
            Operator::I32Const { value } => self.num_const(consts.i32_const(*value as u32)),
            Operator::I64Const { value } => self.num_const(consts.i64_const(*value as u64)),
            Operator::Call { function_index } => self.call(consts.id_const(*function_index as u16)),
            // TODO
            _ => unreachable(),
        }
    }
}

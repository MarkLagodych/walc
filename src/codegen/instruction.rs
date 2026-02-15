mod context;
use context::*;

mod arith;
use arith::*;

use crate::{analyzer::*, codegen::*};

use std::collections::BTreeMap as Map;

pub type Instruction = Expr;

struct DefCtx<'a> {
    consts: &'a mut number::ConstantDefinitionBuilder,
}

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

    pub fn build(self, consts: &mut number::ConstantDefinitionBuilder) -> DefinitionBuilder {
        let mut b = DefinitionBuilder::new();

        // TODO use ArithDefBuilder here
        let mut ctx = DefCtx { consts };

        for (def_name, def) in self.map.into_iter() {
            b.def(def_name, def(&mut ctx));
        }

        b
    }

    pub fn instruction(
        &mut self,
        op: &Operator,
        info: &FunctionInfo,
        consts: &mut number::ConstantDefinitionBuilder,
        labels: &[Expr],
    ) -> Instruction {
        match op {
            Operator::I32Const { value } => self.push(consts.i32_const(*value as u32)),
            Operator::I64Const { value } => self.push(consts.i64_const(*value as u64)),
            Operator::Call { function_index } => self.call(consts.id_const(*function_index as u16)),
            // TODO
            _ => unreachable(),
        }
    }

    pub fn push(&mut self, item: Expr) -> Instruction {
        self.insert("Push", Self::push_def);

        apply(var("Push"), [item])
    }

    fn push_def(_ctx: &mut DefCtx) -> Expr {
        abs(["item"], instruction(|ctx| ctx.push(var("item"))))
    }

    pub fn call(&mut self, function_id: number::Id) -> Instruction {
        self.insert("Call", Self::call_def);

        apply(var("Call"), [function_id])
    }

    fn call_def(_ctx: &mut DefCtx) -> Expr {
        abs(["funcid"], instruction(|ctx| ctx.call(var("funcid"))))
    }

    pub fn enter(
        &mut self,
        func_type: &FuncType,
        local_types: &[ValType],
        consts: &mut number::ConstantDefinitionBuilder,
    ) -> Instruction {
        self.insert("Enter", Self::enter_def);

        apply(var("Enter"), [todo!()])
    }

    fn enter_def(_ctx: &mut DefCtx) -> Expr {
        todo!()
    }
}

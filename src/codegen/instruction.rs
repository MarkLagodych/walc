use super::*;

use crate::analyzer::{Operator, ValType};

use std::collections::BTreeSet as Set;

pub type Instruction = Expr;

type ExecutionContext = Expr;

#[derive(Default)]
struct ExecutionContextBuilder {
    memory: Option<Expr>,
    globals: Option<Expr>,
    locals: Option<Expr>,
    data_stack: Option<Expr>,
    jump_stack: Option<Expr>,
}

impl ExecutionContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn memory(&self) -> memory::Memory {
        var("M")
    }

    pub fn set_memory(&mut self, mem: memory::Memory) {
        self.memory = Some(mem);
    }

    pub fn globals(&self) -> table::Table {
        var("G")
    }

    pub fn set_globals(&mut self, globals: table::Table) {
        self.globals = Some(globals);
    }

    pub fn locals(&self) -> stack::Stack {
        var("L")
    }

    pub fn set_locals(&mut self, locals: stack::Stack) {
        self.locals = Some(locals);
    }

    pub fn data_stack(&self) -> stack::Stack {
        var("D")
    }

    pub fn set_data_stack(&mut self, data_stack: stack::Stack) {
        self.data_stack = Some(data_stack);
    }

    pub fn jump_stack(&self) -> stack::Stack {
        var("J")
    }

    pub fn set_jump_stack(&mut self, jump_stack: stack::Stack) {
        self.jump_stack = Some(jump_stack);
    }

    pub fn build(self) -> ExecutionContext {
        let f = var("F");
        let m = self.memory.unwrap_or_else(|| var("M"));
        let g = self.globals.unwrap_or_else(|| var("G"));
        let l = self.locals.unwrap_or_else(|| var("L"));
        let d = self.data_stack.unwrap_or_else(|| var("D"));
        let j = self.jump_stack.unwrap_or_else(|| var("J"));

        abs(["E"], apply(var("E"), [f, m, g, l, d, j]))
    }
}

fn instr(make_body: impl FnOnce(ExecutionContextBuilder) -> ExecutionContext) -> Instruction {
    abs(
        ["N", "F", "M", "G", "L", "D", "J"],
        make_body(ExecutionContextBuilder::new()),
    )
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
enum InstructionId {
    Const,
    // TODO more instructions
}

#[derive(Default)]
pub struct InstructionDefinitionBuilder {
    instr_set: Set<InstructionId>,
}

impl InstructionDefinitionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    fn _const(&self) -> Expr {
        // TODO handle data stack stack! It's a stack of stacks of data items!
        abs(
            ["item"],
            instr(|mut ctx| {
                let b = DefinitionBuilder::new();
                ctx.set_data_stack(stack::push(ctx.data_stack(), var("item")));
                b.build(ctx.build())
            }),
        )
    }

    pub fn build(self, b: &mut DefinitionBuilder) {
        // TODO use ArithDefBuilder here

        for instr_id in &self.instr_set {
            use InstructionId::*;

            match instr_id {
                Const => b.def("Const", self._const()),
                // TODO
            }
        }
    }

    pub fn instruction(
        &mut self,
        consts: &mut number::ConstantDefinitionBuilder,
        op: &Operator,
    ) -> Instruction {
        use InstructionId::*;

        match op {
            Operator::I32Const { value } => {
                self.instr_set.insert(Const);
                apply(var("Const"), [consts.i32_const(*value as u32)])
            }
            Operator::I64Const { value } => {
                self.instr_set.insert(Const);
                apply(var("Const"), [consts.i64_const(*value as u64)])
            }
            // TODO
            _ => unreachable(),
        }
    }
}

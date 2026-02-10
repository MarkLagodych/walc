use super::*;

use crate::analyzer::*;

use std::collections::BTreeSet as Set;

pub type Instruction = Expr;

type ExecutionContext = Expr;

#[derive(Default)]
struct Ctx {
    memory: Option<memory::Memory>,
    globals: Option<table::Table>,
    locals: Option<stack::Stack>,
    data_stack: Option<framed_stack::FramedStack>,
    jump_stack: Option<stack::Stack>,
}

impl Ctx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn function_table(&self) -> table::Table {
        pair::get_first(var("F"))
    }

    pub fn indirect_function_table(&self) -> table::Table {
        pair::get_second(var("F"))
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

    pub fn data_stack(&self) -> framed_stack::FramedStack {
        var("D")
    }

    pub fn set_data_stack(&mut self, data_stack: framed_stack::FramedStack) {
        self.data_stack = Some(data_stack);
    }

    pub fn jump_stack(&self) -> stack::Stack {
        var("J")
    }

    pub fn set_jump_stack(&mut self, jump_stack: stack::Stack) {
        self.jump_stack = Some(jump_stack);
    }

    pub fn get_next(&self) -> Expr {
        var("N")
    }

    pub fn next(self) -> ExecutionContext {
        let next = self.get_next();
        self.jump(next)
    }

    pub fn jump(self, target: Expr) -> ExecutionContext {
        let f = var("F");
        let m = self.memory.unwrap_or_else(|| var("M"));
        let g = self.globals.unwrap_or_else(|| var("G"));
        let l = self.locals.unwrap_or_else(|| var("L"));
        let d = self.data_stack.unwrap_or_else(|| var("D"));
        let j = self.jump_stack.unwrap_or_else(|| var("J"));

        apply(target, [f, m, g, l, d, j])
    }
}

fn instr(body: Expr) -> Instruction {
    abs(["N", "F", "M", "G", "L", "D", "J"], body)
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
enum InstructionId {
    Const,
    Call,
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
        abs(
            ["item"],
            instr({
                let mut ctx = Ctx::new();
                ctx.set_data_stack(framed_stack::push(ctx.data_stack(), var("item")));
                ctx.next()
            }),
        )
    }

    fn _call(&self) -> Expr {
        abs(
            ["func"],
            instr({
                let mut ctx = Ctx::new();
                ctx.set_jump_stack(stack::push(ctx.jump_stack(), ctx.get_next()));
                let func = table::index(ctx.function_table(), var("func"));
                ctx.jump(func)
            }),
        )
    }

    pub fn build(self, b: &mut DefinitionBuilder) {
        // TODO use ArithDefBuilder here
        // TODO can constant builder be needed here???

        for instr_id in &self.instr_set {
            use InstructionId::*;

            match instr_id {
                Const => b.def("Const", self._const()),
                Call => b.def("Call", self._call()),
                // TODO
            }
        }
    }

    pub fn instruction(
        &mut self,
        consts: &mut number::ConstantDefinitionBuilder,
        function_types: &[FuncType],
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
            Operator::Call { function_index } => {
                self.instr_set.insert(Call);
                apply(var("Call"), [consts.id_const(*function_index as u16)])
            }
            // TODO
            _ => unreachable(),
        }
    }
}

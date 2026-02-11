use crate::codegen::*;

use instruction::Instruction;

pub type ExecutionContext = Expr;

#[derive(Default)]
pub struct ContextBuilder {
    next: Option<Instruction>,
    memory: Option<memory::Memory>,
    globals: Option<table::Table>,
    trace: Option<stack::Stack>,
    locals: Option<framed_table::FramedTable>,
    stack: Option<framed_stack::FramedStack>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next(mut self, next: Instruction) -> Self {
        self.next = Some(next);
        self
    }

    pub fn memory(mut self, mem: memory::Memory) -> Self {
        self.memory = Some(mem);
        self
    }

    pub fn globals(mut self, globals: table::Table) -> Self {
        self.globals = Some(globals);
        self
    }

    pub fn locals(mut self, locals: framed_table::FramedTable) -> Self {
        self.locals = Some(locals);
        self
    }

    pub fn stack(mut self, stack: framed_stack::FramedStack) -> Self {
        self.stack = Some(stack);
        self
    }

    pub fn trace(mut self, trace: stack::Stack) -> Self {
        self.trace = Some(trace);
        self
    }

    pub fn build(self) -> ExecutionContext {
        let n = self.next.unwrap_or_else(|| next());
        let f = function_tables();
        let m = self.memory.unwrap_or_else(|| memory());
        let g = self.globals.unwrap_or_else(|| globals());
        let l = self.locals.unwrap_or_else(|| locals());
        let s = self.stack.unwrap_or_else(|| stack());
        let t = self.trace.unwrap_or_else(|| trace());

        apply(n, [f, m, g, l, s, t])
    }
}

fn function_tables() -> pair::Pair {
    var("F")
}

pub fn functions() -> table::Table {
    pair::get_first(function_tables())
}

pub fn indirect_function_table() -> table::Table {
    pair::get_second(function_tables())
}

pub fn next() -> Instruction {
    var("N")
}

pub fn memory() -> memory::Memory {
    var("M")
}

pub fn globals() -> table::Table {
    var("G")
}

pub fn locals() -> stack::Stack {
    var("L")
}

pub fn stack() -> framed_stack::FramedStack {
    var("S")
}

pub fn trace() -> stack::Stack {
    var("T")
}

pub fn instr(body: Expr) -> Instruction {
    abs(["N", "F", "M", "G", "L", "S", "T"], body)
}

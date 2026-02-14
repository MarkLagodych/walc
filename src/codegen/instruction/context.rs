use crate::codegen::*;

use instruction::Instruction;

pub struct CurrentContext {}
pub type NewContext = Expr;

pub fn instruction(body_fn: impl FnOnce(&CurrentContext) -> NewContext) -> Instruction {
    abs(
        ["N", "F", "M", "G", "L", "S", "T"],
        body_fn(&CurrentContext {}),
    )
}

impl CurrentContext {
    pub fn next(&self) -> Instruction {
        var("N")
    }

    pub fn memory(&self) -> memory::Memory {
        var("M")
    }

    pub fn globals(&self) -> table::Table {
        var("G")
    }

    pub fn locals(&self) -> locals::Locals {
        var("L")
    }

    pub fn stack(&self) -> data_stack::DataStack {
        var("S")
    }

    pub fn trace(&self) -> stack::Stack {
        var("T")
    }

    fn function_tables(&self) -> pair::Pair {
        var("F")
    }

    pub fn function_table(&self) -> table::Table {
        pair::get_first(self.function_tables())
    }

    pub fn indirect_function_table(&self) -> table::Table {
        pair::get_second(self.function_tables())
    }
}

pub struct NewContextBuilder {
    next: Instruction,
    memory: memory::Memory,
    globals: table::Table,
    locals: locals::Locals,
    stack: data_stack::DataStack,
    trace: stack::Stack,
}

impl NewContextBuilder {
    pub fn new() -> Self {
        Self {
            next: var("N"),
            memory: var("M"),
            globals: var("G"),
            locals: var("L"),
            stack: var("S"),
            trace: var("T"),
        }
    }

    pub fn next(mut self, next: Instruction) -> Self {
        self.next = next;
        self
    }

    pub fn memory(mut self, memory: memory::Memory) -> Self {
        self.memory = memory;
        self
    }

    pub fn globals(mut self, globals: table::Table) -> Self {
        self.globals = globals;
        self
    }

    pub fn locals(mut self, locals: locals::Locals) -> Self {
        self.locals = locals;
        self
    }

    pub fn stack(mut self, stack: data_stack::DataStack) -> Self {
        self.stack = stack;
        self
    }

    pub fn trace(mut self, trace: stack::Stack) -> Self {
        self.trace = trace;
        self
    }

    pub fn build(self) -> NewContext {
        apply(
            self.next,
            [
                var("F"),
                self.memory,
                self.globals,
                self.locals,
                self.stack,
                self.trace,
            ],
        )
    }
}

pub mod locals {
    use super::*;

    /// Stack of tables
    pub type Locals = stack::Stack;

    pub fn new() -> Locals {
        stack::empty()
    }

    pub fn push_frame(locals: Locals, items: impl IntoIterator<Item = Expr>) -> Locals {
        stack::push(locals, table::from(items))
    }

    pub fn pop_frame(locals: Locals) -> Locals {
        stack::pop(locals)
    }

    pub fn index(locals: Locals, local_id: number::Id) -> Expr {
        table::index(stack::top(locals), local_id)
    }

    pub fn insert(locals: Locals, local_id: number::Id, value: Expr) -> Locals {
        let top_table = stack::top(locals.clone());
        let new_top = table::insert(top_table, local_id, value);
        stack::push(stack::pop(locals), new_top)
    }
}

pub mod data_stack {
    use super::*;

    /// Stack of stacks.
    /// Every substack represents a call/block frame.
    pub type DataStack = stack::Stack;

    pub fn empty() -> DataStack {
        stack::empty()
    }

    pub fn push_frame(stack: DataStack) -> DataStack {
        stack::push(stack, stack::empty())
    }

    pub fn pop_frame(stack: DataStack) -> DataStack {
        stack::pop(stack)
    }

    pub fn push(stack: DataStack, item: Expr) -> DataStack {
        let top_stack = stack::top(stack.clone());
        let new_top = stack::push(top_stack, item);
        stack::push(stack::pop(stack), new_top)
    }

    pub fn top(stack: DataStack) -> Expr {
        stack::top(stack::top(stack))
    }

    pub fn pop(stack: DataStack) -> DataStack {
        let top_stack = stack::top(stack.clone());
        let new_top = stack::pop(top_stack);
        stack::push(stack::pop(stack), new_top)
    }
}

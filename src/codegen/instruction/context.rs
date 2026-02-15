use crate::codegen::*;

use instruction::Instruction;

pub fn instruction(body: impl FnOnce(&mut InstructionContext)) -> Instruction {
    let mut ctx = InstructionContext::new();

    body(&mut ctx);

    abs(["N", "F", "M", "G", "L", "S", "T"], ctx.build())
}

#[derive(Default)]
pub struct InstructionContext {
    defs: DefinitionBuilder,
}

impl InstructionContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Expr {
        self.defs.build(apply(
            var("N"),
            [var("F"), var("M"), var("G"), var("L"), var("S"), var("T")],
        ))
    }

    pub fn next(&self) -> Instruction {
        var("N")
    }

    pub fn set_next(&mut self, next: Expr) {
        self.def("N", next);
    }

    pub fn memory(&self) -> memory::Memory {
        var("M")
    }

    pub fn set_memory(&mut self, memory: Expr) {
        self.def("M", memory);
    }

    pub fn globals(&self) -> table::Table {
        var("G")
    }

    pub fn set_globals(&mut self, globals: Expr) {
        self.def("G", globals);
    }

    pub fn locals(&self) -> locals::Locals {
        var("L")
    }

    pub fn set_locals(&mut self, locals: Expr) {
        self.def("L", locals);
    }

    pub fn stack(&self) -> data_stack::DataStack {
        var("S")
    }

    pub fn set_stack(&mut self, stack: Expr) {
        self.def("S", stack);
    }

    pub fn trace(&self) -> stack::Stack {
        var("T")
    }

    pub fn set_trace(&mut self, trace: Expr) {
        self.def("T", trace);
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

    pub fn def(&mut self, name: impl ToString, def: Expr) {
        self.defs.def(name, def);
    }

    pub fn push(&mut self, item: Expr) {
        self.set_stack(data_stack::push(self.stack(), item));
    }

    pub fn pop(&mut self, dest_var: impl ToString) {
        self.def(dest_var, data_stack::top(self.stack()));
        self.set_stack(data_stack::pop(self.stack()));
    }

    pub fn get_top(&mut self, dest_var: impl ToString) {
        self.def(dest_var, data_stack::top(self.stack()));
    }

    pub fn push_trace(&mut self, item: Expr) {
        self.set_trace(stack::push(self.trace(), item));
    }

    pub fn pop_trace(&mut self, dest_var: impl ToString) {
        self.def(dest_var, stack::top(self.trace()));
        self.set_trace(stack::pop(self.trace()));
    }

    pub fn get_global(&mut self, dest_var: impl ToString, global_id: number::Id) {
        self.def(dest_var, table::index(self.globals(), global_id));
    }

    pub fn set_global(&mut self, global_id: number::Id, value: Expr) {
        self.set_globals(table::insert(self.globals(), global_id, value));
    }

    pub fn get_local(&mut self, dest_var: impl ToString, local_id: number::Id) {
        self.def(dest_var, locals::index(self.locals(), local_id));
    }

    pub fn set_local(&mut self, local_id: number::Id, value: Expr) {
        self.set_locals(locals::insert(self.locals(), local_id, value));
    }

    pub fn load(&mut self, dest_var: impl ToString, address: number::I32) {
        self.def(dest_var, memory::index(self.memory(), address));
    }

    pub fn store(&mut self, address: number::I32, value: Expr) {
        self.set_memory(memory::insert(self.memory(), address, value));
    }

    pub fn get_function(&mut self, dest_var: impl ToString, function_id: number::Id) {
        self.def(dest_var, table::index(self.function_table(), function_id));
    }

    pub fn get_function_indirect(&mut self, dest_var: impl ToString, function_id: number::Id) {
        self.def(
            dest_var,
            table::index(self.indirect_function_table(), function_id),
        );
    }

    pub fn call(&mut self, function_id: number::Id) {
        self.push_trace(self.next());
        self.set_next(table::index(self.function_table(), function_id));
    }

    pub fn call_indirect(&mut self, indirect_id: number::Id) {
        self.push_trace(self.next());
        self.set_next(table::index(self.indirect_function_table(), indirect_id));
    }

    pub fn new_frame(&mut self, trace: Expr, locals: impl IntoIterator<Item = Expr>) {
        self.push_trace(trace);
        self.set_locals(locals::push_frame(self.locals(), locals));
        self.set_stack(data_stack::push_frame(self.stack()));
    }

    pub fn pop_frame(&mut self) {
        self.set_locals(locals::pop_frame(self.locals()));
        self.set_stack(data_stack::pop_frame(self.stack()));
        self.pop_trace("a");
        self.set_next(var("a"));
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

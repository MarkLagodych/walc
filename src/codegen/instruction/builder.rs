use crate::codegen::*;

use instruction::Instruction;

enum IoOptions {
    Output(number::Byte),
    Input(String),
    Exit,
}

#[derive(Default)]
pub struct InstructionBuilder {
    defs: DefinitionBuilder,
    io_options: Option<IoOptions>,
}

impl InstructionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn def(&mut self, name: impl ToString, value: Expr) {
        self.defs.def(name, value);
    }

    pub fn build(self) -> io_command::IoCommand {
        let next = apply(
            self.next(),
            [
                self.function_tables(),
                self.memory(),
                self.globals(),
                self.locals(),
                self.stack(),
                self.trace(),
            ],
        );

        let result = match self.io_options {
            Some(IoOptions::Output(byte)) => self.defs.build(io_command::output(byte, next)),
            Some(IoOptions::Input(dest_var)) => {
                io_command::input(abs([dest_var], self.defs.build(next)))
            }
            Some(IoOptions::Exit) => io_command::exit(),
            None => self.defs.build(next),
        };

        abs(["N", "F", "M", "G", "L", "S", "T"], result)
    }

    /// Makes the resulting instruction exit the program
    pub fn set_exit(&mut self) {
        self.io_options = Some(IoOptions::Exit);
    }

    /// Makes the resulting instruction write the given byte to the output *after* performing
    /// all other operations inside itself.
    pub fn set_output(&mut self, byte: number::Byte) {
        self.io_options = Some(IoOptions::Output(byte));
    }

    /// Makes the resulting instruction read an input byte and store it in the given variable
    /// *before* performing all other operations inside itself.
    pub fn set_input(&mut self, dest_var: impl ToString) {
        self.io_options = Some(IoOptions::Input(dest_var.to_string()));
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

    pub fn push(&mut self, items: impl IntoIterator<Item = Expr>) {
        // Items are always pushed left-to-right in WASM
        for item in items {
            self.set_stack(data_stack::push(self.stack(), item));
        }
    }

    pub fn pop<ToStr, DestVars>(&mut self, dest_vars: DestVars)
    where
        ToStr: ToString,
        DestVars: IntoIterator<Item = ToStr>,
        DestVars::IntoIter: DoubleEndedIterator<Item = ToStr>,
    {
        // Pop right-to-left
        for dest_var in dest_vars.into_iter().rev() {
            self.def(dest_var.to_string(), data_stack::top(self.stack()));
            self.drop();
        }
    }

    pub fn drop(&mut self) {
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

    pub fn ret(&mut self) {
        self.pop_trace("_ret");
        self.set_next(var("_ret"));
    }

    pub fn push_frame(&mut self, locals: table::Table) {
        self.set_locals(locals::push_frame(self.locals(), locals));
        self.set_stack(data_stack::push_frame(self.stack()));
    }

    pub fn pop_frame(&mut self) {
        self.set_locals(locals::pop_frame(self.locals()));
        self.set_stack(data_stack::pop_frame(self.stack()));
    }
}

pub mod locals {
    use super::*;

    /// Stack of tables.
    /// Every table represents a call frame and contains locals of the corresponding function.
    pub type Locals = stack::Stack;

    pub fn new() -> Locals {
        stack::empty()
    }

    pub fn push_frame(locals: Locals, items: table::Table) -> Locals {
        stack::push(locals, items)
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
    /// Every substack represents a call *or* a block frame.
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

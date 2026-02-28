use super::*;

/// An instruction is a function of `(N, F, M, G, L, S, T) -> IoCommand` where:
/// - `N` is the next code segment (or `unreachable` is this is the last instruction in a function).
///   See [`code::Code`].
/// - `F` is a pair of the global function table and the user-defined table for indirect calls
/// - `M` is the memory. See [`memory::Memory`].
/// - `G` is the global variable table. See [`table::Table`].
/// - `L` is the stack of local variable tables. See [`locals::Locals`].
/// - `S` is the data stack. See [`data_stack::DataStack`].
/// - `T` is the trace (i.e. control flow stack).
///   This is only used for jumps directed backwards (`loop`) and function calls.
///   This is a normal stack (see [`stack::Stack`]) that stores code segments where to return to.
///
/// See also [`io_command::IoCommand`].
///
/// An instruction might return an `IoCommand` directly or by invoking the next instruction
/// (or any other instruction in general).
pub type Instruction = Expr;

/// Starts program execution at the given start function and with the given environment.
pub fn start(
    start_function: code::Code,
    function_table: table::Table,
    custom_table: table::Table,
    globals: table::Table,
) -> io_command::IoCommand {
    apply(
        start_function,
        [
            pair::new(function_table, custom_table),
            memory::new(),
            globals,
            locals::new(),
            data_stack::empty(),
            stack::empty(),
        ],
    )
}

pub fn nop() -> Instruction {
    abs(["nop"], var("nop"))
}

#[derive(Default)]
pub struct InstructionBuilder {
    defs: LetExprBuilder,
}

impl InstructionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn def(&mut self, name: impl ToString, value: Expr) {
        self.defs.def(name, value);
    }

    /// Builds a normal instruction that does not perform any I/O itself.
    pub fn build(self) -> io_command::IoCommand {
        let next = self.execute_next();
        let body = self.defs.build_in(next);
        Self::wrap_with_context(body)
    }

    /// Builds an instruction that unconditionally exits the program.
    /// The instruction body is discarded.
    pub fn build_exit(self) -> io_command::IoCommand {
        Self::wrap_with_context(io_command::exit())
    }

    /// Builds an instruction that writes the given byte to the output *after* performing
    /// all other operations inside itself.
    pub fn build_output(self, byte: number::Byte) -> io_command::IoCommand {
        let next = self.execute_next();
        let cmd = io_command::output(byte, next);
        let body = self.defs.build_in(cmd);
        Self::wrap_with_context(body)
    }

    /// Builds an instruction that reads an input byte and stores it into the given variable
    /// *before* performing all other operations inside itself.
    pub fn build_input(self, dest_var: impl ToString) -> io_command::IoCommand {
        let next = self.execute_next();
        let body = self.defs.build_in(next);
        let cmd = io_command::input(abs([dest_var.to_string()], body));
        Self::wrap_with_context(cmd)
    }

    fn execute_next(&self) -> io_command::IoCommand {
        apply(
            self.next(),
            [
                self.function_tables(),
                self.memory(),
                self.globals(),
                self.locals(),
                self.stack(),
                self.trace(),
            ],
        )
    }

    fn wrap_with_context(body: Expr) -> Instruction {
        abs(["N", "F", "M", "G", "L", "S", "T"], body)
    }

    pub fn next(&self) -> code::Code {
        var("N")
    }

    pub fn set_next(&mut self, next: code::Code) {
        self.def("N", next);
    }

    pub fn memory(&self) -> memory::Memory {
        var("M")
    }

    pub fn set_memory(&mut self, memory: memory::Memory) {
        self.def("M", memory);
    }

    pub fn globals(&self) -> table::Table {
        var("G")
    }

    pub fn set_globals(&mut self, globals: table::Table) {
        self.def("G", globals);
    }

    pub fn locals(&self) -> locals::Locals {
        var("L")
    }

    pub fn set_locals(&mut self, locals: locals::Locals) {
        self.def("L", locals);
    }

    pub fn stack(&self) -> data_stack::DataStack {
        var("S")
    }

    pub fn set_stack(&mut self, stack: data_stack::DataStack) {
        self.def("S", stack);
    }

    pub fn trace(&self) -> stack::Stack {
        var("T")
    }

    pub fn set_trace(&mut self, trace: stack::Stack) {
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

    pub fn get_trace_top(&mut self, dest_var: impl ToString) {
        self.def(dest_var, stack::top(self.trace()));
    }

    pub fn drop_trace(&mut self) {
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

    pub fn push_locals_frame(&mut self, locals: table::Table) {
        self.set_locals(locals::push_frame(self.locals(), locals));
        self.set_stack(data_stack::push_frame(self.stack()));
    }

    pub fn pop_locals_frame(&mut self) {
        self.set_locals(locals::pop_frame(self.locals()));
        self.set_stack(data_stack::pop_frame(self.stack()));
    }

    pub fn push_stack_frame(&mut self) {
        self.set_stack(data_stack::push_frame(self.stack()));
    }

    pub fn pop_stack_frame(&mut self) {
        self.set_stack(data_stack::pop_frame(self.stack()));
    }
}

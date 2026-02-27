use super::*;

pub mod unsafe_list {
    use super::*;

    pub type UnsafeList = Expr;

    pub fn empty() -> Expr {
        unreachable()
    }

    pub fn node(head: Expr, tail: Expr) -> UnsafeList {
        pair::new(head, tail)
    }

    pub fn get_head(list: UnsafeList) -> Expr {
        pair::get_first(list)
    }

    pub fn get_tail(list: UnsafeList) -> Expr {
        pair::get_second(list)
    }
}

pub mod list {
    use super::*;

    pub type List = Expr;

    pub fn empty() -> List {
        var("Empty")
    }

    pub fn node(head: Expr, tail: Expr) -> List {
        apply(var("C"), [head, tail])
    }

    pub fn from(items: impl DoubleEndedIterator<Item = Expr>) -> List {
        let mut result = empty();
        for item in items.rev() {
            result = node(item, result);
        }
        result
    }

    pub fn is_not_empty(list: List) -> Expr {
        optional::is_some(list)
    }

    pub fn get_head(list: List) -> Expr {
        pair::get_first(optional::unwrap(list))
    }

    pub fn get_tail(list: List) -> Expr {
        pair::get_second(optional::unwrap(list))
    }

    pub fn generate_defs(b: &mut LetExprBuilder) {
        b.def("Empty", optional::none());

        b.def(
            // Cons (list constructor)
            "C",
            abs(
                ["head", "tail"],
                optional::some(pair::new(var("head"), var("tail"))),
            ),
        );
    }
}

pub mod stack {
    pub use super::*;

    pub type Stack = unsafe_list::UnsafeList;

    pub fn empty() -> Stack {
        unsafe_list::empty()
    }

    pub fn push(stack: Stack, item: Expr) -> Stack {
        unsafe_list::node(item, stack)
    }

    pub fn top(stack: Stack) -> Expr {
        unsafe_list::get_head(stack)
    }

    pub fn pop(stack: Stack) -> Stack {
        unsafe_list::get_tail(stack)
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

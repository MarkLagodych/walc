use super::*;

/// For simplicity and efficient memory usage, untouched subtrees are initialized with dummies.
/// Example of a 5-bit tree (max. 32 values) with 5 items:
/// ```text
///
///  A B C D E (dummy tree with depth 0, 1 dummy item)
///  \/  \/  \/  (dummy tree with depth 1, 2 dummy items)
///   \  /    \ /
///    \/     |/   <- these nodes have level 2
///     \    /
///      \  /
///       \/ (dummy tree with depth 3, 8 dummy items)
///        \/ (dummy tree with depth 4, 16 dummy items)
///         \/
///        Tree
/// ```
///
/// All leaves of dummy trees are implicitly initialized with the null byte.
/// This is because the only place where the default item matters is the main memory, which is
/// zero-initialized in WASM.
pub mod tree {
    use super::*;

    pub type Tree = Expr;

    pub fn new(bitness: u8) -> Tree {
        debug_assert!(bitness <= 32);
        var(format!("T{bitness}"))
    }

    pub fn node(left: Expr, right: Expr) -> Tree {
        pair::new(left, right)
    }

    pub fn get_left(tree: Tree) -> Expr {
        pair::get_first(tree)
    }

    pub fn get_right(tree: Tree) -> Expr {
        pair::get_second(tree)
    }

    pub fn select_subtree(tree: Tree, selector: Bit) -> Expr {
        pair::select(tree, selector)
    }

    /// Index the tree with a little-endian number (e.g. `I32`).
    /// The index bitness must match the tree bitness.
    pub fn index_num(tree: Tree, index: number::Number) -> Expr {
        apply(var("IdxLE"), [tree, index])
    }

    /// Index the tree with an ID.
    /// The index bitness must match the tree bitness.
    pub fn index_id(tree: Tree, index: number::Id) -> Expr {
        apply(var("IdxBE"), [tree, index])
    }

    /// The index bitness must match the tree bitness.
    pub fn insert_num(tree: Tree, index: number::Number, value: Expr) -> Tree {
        apply(var("InsLE"), [tree, index, value])
    }

    pub fn insert_id(tree: Tree, index: number::Id, value: Expr) -> Tree {
        apply(var("InsBE"), [tree, index, value])
    }

    /// The index bitness must match the tree bitness.
    pub fn from(bitness: u8, items: impl IntoIterator<Item = Expr>) -> Tree {
        debug_assert!(bitness <= 32);

        let mut items = items.into_iter().collect::<Vec<Expr>>();

        if items.is_empty() {
            return tree::new(bitness);
        }

        for i in 0..bitness {
            if items.len() % 2 != 0 {
                items.push(tree::new(i));
            }

            items = items
                .chunks(2)
                .map(|chunk| tree::node(chunk[0].clone(), chunk[1].clone()))
                .collect();
        }

        items[0].clone()
    }

    pub fn generate_defs(b: &mut LetExprBuilder) {
        // Dummy trees

        // Indexable by 0 bits (i.e. not indexable)
        b.def("T0", number::null_byte());
        // Every node is indexable by i bits
        for i in 1..=32 {
            b.def(
                format!("T{i}"),
                tree::node(var(format!("T{}", i - 1)), var(format!("T{}", i - 1))),
            )
        }

        b.def_rec(
            "IdxBE_",
            abs(["tree", "index"], {
                select(list::is_not_empty(var("index")), var("tree"), {
                    let index_bit = list::get_head(var("index"));
                    let index_tail = list::get_tail(var("index"));
                    let subtree = tree::select_subtree(var("tree"), index_bit);
                    apply(rec(var("IdxBE_")), [subtree, index_tail])
                })
            }),
        );

        b.def("IdxBE", rec(var("IdxBE_")));

        b.def(
            "IdxLE",
            abs(
                ["tree", "index"],
                apply(
                    rec(var("IdxBE_")),
                    [var("tree"), number::reverse_bits(var("index"))],
                ),
            ),
        );

        b.def_rec(
            "InsBE_",
            abs(["tree", "index", "value"], {
                let left = tree::get_left(var("tree"));
                let right = tree::get_right(var("tree"));

                let index_bit = list::get_head(var("index"));
                let index_tail = list::get_tail(var("index"));

                let insert = |subtree| {
                    apply(
                        rec(var("InsBE_")),
                        [subtree, index_tail.clone(), var("value")],
                    )
                };

                select(
                    list::is_not_empty(var("index")),
                    var("value"),
                    select(
                        index_bit,
                        tree::node(insert(left.clone()), right.clone()),
                        tree::node(left.clone(), insert(right.clone())),
                    ),
                )
            }),
        );

        b.def("InsBE", rec(var("InsBE_")));

        b.def(
            "InsLE",
            abs(
                ["tree", "index", "value"],
                apply(
                    rec(var("InsBE_")),
                    [
                        var("tree"),
                        number::reverse_bits(var("index")),
                        var("value"),
                    ],
                ),
            ),
        );
    }
}

pub mod memory {
    use super::*;

    /// Zero-initialized, indexed by 32 bits.
    pub type Memory = tree::Tree;

    pub fn new() -> Memory {
        tree::new(32)
    }

    pub fn index(memory: Memory, address: number::I32) -> Expr {
        tree::index_num(memory, address)
    }

    pub fn insert(memory: Memory, address: number::I32, value: Expr) -> Memory {
        tree::insert_num(memory, address, value)
    }
}

pub mod table {
    use super::*;

    /// Used for storing functions, globals, and local frames.
    /// Uses faster 16-bit indexing because 65536 entities should be more than enough
    /// for this project.
    pub type Table = tree::Tree;

    pub fn from(items: impl IntoIterator<Item = Expr>) -> Table {
        tree::from(16, items)
    }

    pub fn index(table: Table, address: number::Id) -> Expr {
        tree::index_id(table, address)
    }

    pub fn insert(table: Table, address: number::Id, value: Expr) -> Table {
        tree::insert_id(table, address, value)
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

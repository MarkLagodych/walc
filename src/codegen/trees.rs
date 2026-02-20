//! All dummy trees are implicitly initialized with the null byte.
//! This is because the only place where the default item matters is the main memory, which is
//! zero-initialized in WASM.

use super::*;

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

    /// The index bitness must match the tree bitness.
    pub fn index(tree: Tree, index: number::Number) -> Expr {
        apply(var("Idx"), [tree, index])
    }

    /// The index bitness must match the tree bitness.
    pub fn insert(tree: Tree, index: number::Number, value: Expr) -> Tree {
        apply(var("Ins"), [tree, index, value])
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

    pub fn define_prelude(b: &mut DefinitionBuilder) {
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

        b.def(
            "Idx_",
            abs(["idx", "tree", "index"], {
                select(list::is_not_empty(var("index")), var("tree"), {
                    let index_bit = list::get_head(var("index"));
                    let index_tail = list::get_tail(var("index"));
                    let subtree = tree::select_subtree(var("tree"), index_bit);
                    apply(rec(var("idx")), [subtree, index_tail])
                })
            }),
        );

        b.def(
            "Idx",
            abs(
                ["tree", "index"],
                apply(
                    rec(var("Idx_")),
                    [var("tree"), number::reverse_bits(var("index"))],
                ),
            ),
        );

        b.def(
            "Ins_",
            abs(["ins", "tree", "index", "value"], {
                let left = tree::get_left(var("tree"));
                let right = tree::get_right(var("tree"));

                let index_bit = list::get_head(var("index"));
                let index_tail = list::get_tail(var("index"));

                let insert =
                    |subtree| apply(rec(var("ins")), [subtree, index_tail.clone(), var("value")]);

                select(
                    list::is_not_empty(var("index")),
                    var("tree"),
                    select(
                        index_bit,
                        tree::node(insert(left.clone()), right.clone()),
                        tree::node(left.clone(), insert(right.clone())),
                    ),
                )
            }),
        );

        b.def(
            "Ins",
            abs(
                ["tree", "index", "value"],
                apply(
                    rec(var("Ins_")),
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

    pub type Memory = tree::Tree;

    pub fn new() -> Memory {
        tree::new(32)
    }

    pub fn index(memory: Memory, address: number::I32) -> Expr {
        tree::index(memory, address)
    }

    pub fn insert(memory: Memory, address: number::I32, value: Expr) -> Memory {
        tree::insert(memory, address, value)
    }
}

pub mod table {
    use super::*;

    pub type Table = tree::Tree;

    pub fn from(items: impl IntoIterator<Item = Expr>) -> Table {
        tree::from(16, items)
    }

    pub fn index(table: Table, address: number::Id) -> Expr {
        tree::index(table, address)
    }

    pub fn insert(table: Table, address: number::Id, value: Expr) -> Table {
        tree::insert(table, address, value)
    }
}

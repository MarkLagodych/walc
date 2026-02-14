use super::*;

pub mod tree {
    use super::*;

    pub type Tree = Expr;

    pub fn new(bitness: u8, initial_item: Expr) -> Tree {
        debug_assert!(bitness <= 32);
        apply(var(format!("Tr{bitness}")), [initial_item])
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

    /// The index bitness must match the tree bitness.
    pub fn index(tree: Tree, index: number::Number) -> Expr {
        apply(index, [tree])
    }

    /// The index bitness must match the tree bitness.
    pub fn insert(bitness: u8, tree: Tree, index: number::Number, value: Expr) -> Tree {
        debug_assert!(bitness <= 32);
        apply(index, [apply(var(format!("Ins{bitness}")), [tree, value])])
    }

    /// The index bitness must match the tree bitness.
    pub fn from(bitness: u8, items: impl IntoIterator<Item = Expr>, default_item: Expr) -> Tree {
        debug_assert!(bitness <= 32);

        let mut items = items.into_iter().collect::<Vec<Expr>>();

        if items.is_empty() {
            return tree::new(bitness, default_item);
        }

        for i in 0..bitness {
            if items.len() % 2 != 0 {
                items.push(tree::new(i, default_item.clone()));
            }

            items = items
                .chunks(2)
                .map(|chunk| tree::node(chunk[0].clone(), chunk[1].clone()))
                .collect();
        }

        items[0].clone()
    }

    pub fn define_prelude(b: &mut DefinitionBuilder) {
        // Indexable by 0 bits (i.e. not indexable)
        b.def("Tr0", abs(["x"], var("x")));
        // Every node is indexable by i bits
        for i in 1..=32 {
            let node_name = format!("Tr{}", i);
            let item_name = format!("Tr{}", i - 1);
            let item = apply(var(item_name), [var("x")]);
            b.def(node_name, abs(["x"], tree::node(item.clone(), item)));
        }

        b.def(
            "Ins",
            abs(["insert", "tree", "value", "index_bit"], {
                let left = tree::get_left(var("tree"));
                let right = tree::get_right(var("tree"));

                let insert = |subtree| apply(var("insert"), [subtree, var("value")]);

                select(
                    var("index_bit"),
                    tree::node(insert(left.clone()), right.clone()),
                    tree::node(left.clone(), insert(right.clone())),
                )
            }),
        );

        b.def("Ins0", abs(["tree", "value"], var("value")));

        // Each insertion function consumes i bits of the index
        for i in 1..=32 {
            b.def(
                format!("Ins{}", i),
                apply(var("Ins"), [var(format!("Ins{}", i - 1))]),
            );
        }
    }
}

pub mod memory {
    use super::*;

    const BITNESS: u8 = 32;

    pub type Memory = tree::Tree;

    pub fn new() -> Memory {
        tree::new(BITNESS, number::null_byte())
    }

    pub fn index(memory: Memory, address: number::I32) -> Expr {
        tree::index(memory, address)
    }

    pub fn insert(memory: Memory, address: number::I32, value: Expr) -> Memory {
        tree::insert(BITNESS, memory, address, value)
    }
}

pub mod table {
    use super::*;

    const BITNESS: u8 = 16;

    pub type Table = tree::Tree;

    pub fn from(items: impl IntoIterator<Item = Expr>) -> Table {
        tree::from(BITNESS, items, unreachable())
    }

    pub fn index(table: Table, address: number::Id) -> Expr {
        tree::index(table, address)
    }

    pub fn insert(table: Table, address: number::Id, value: Expr) -> Table {
        tree::insert(BITNESS, table, address, value)
    }
}

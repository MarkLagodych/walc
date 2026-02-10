use super::*;

pub mod tree {
    use super::*;

    pub type Tree = Expr;

    pub fn new(bitness: u8, initial_item: Expr) -> Tree {
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
    pub fn index(tree: Tree, index: Expr) -> Expr {
        apply(index, [tree])
    }

    /// The index bitness must match the tree bitness.
    pub fn insert(bitness: u8, tree: Tree, index: number::Number, value: Expr) -> Tree {
        apply(
            var(format!("TIns{bitness}")),
            [tree, number::to_bit_list_be(bitness, index), value],
        )
    }

    // TODO Is this useful? The idea is to generate a bit list at compile time if possible
    // pub fn static_insert(bitness: u8, array: Expr, index: u32, value: Expr) -> Expr {
    //     debug_assert!(bitness <= 32);
    //     insert(bitness, array, todo!(), value)
    // }

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
        b.def("Tr0", abs(["init"], var("init")));
        // Every node is indexable by i bits
        for i in 1..=32 {
            let node_name = format!("Tr{}", i);
            let item_name = format!("Tr{}", i - 1);
            let item = apply(var(item_name), [var("init")]);
            b.def(node_name, abs(["init"], tree::node(item.clone(), item)));
        }

        b.def(
            "TIns",
            abs(["insert", "array", "index", "value"], {
                let index_bit = unsafe_list::get_head(var("index"));
                let index_rest = unsafe_list::get_tail(var("index"));

                let left = tree::get_left(var("array"));
                let right = tree::get_right(var("array"));

                let insert_into =
                    |subtree| apply(var("insert"), [subtree, index_rest.clone(), var("value")]);

                cond(
                    index_bit,
                    tree::node(left.clone(), insert_into(right.clone())),
                    tree::node(insert_into(left.clone()), right.clone()),
                )
            }),
        );

        b.def("TIns0", abs(["array", "index", "value"], var("value")));

        // Each insertion function consumes i bits of the index
        for i in 1..=32 {
            b.def(
                format!("TIns{}", i),
                apply(var("TIns"), [var(format!("TIns{}", i - 1))]),
            );
        }
    }
}

pub mod memory {
    use super::*;

    const BITNESS: u8 = 32;

    pub type Memory = tree::Tree;

    pub fn new(initial_item: Expr) -> Memory {
        tree::new(BITNESS, initial_item)
    }

    pub fn index(memory: Memory, address: number::I32) -> Expr {
        tree::index(memory, address)
    }

    pub fn insert(memory: Memory, address: number::I32, value: Expr) -> Memory {
        tree::insert(BITNESS, memory, address, value)
    }

    // TODO algorithm for fast init from a list
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

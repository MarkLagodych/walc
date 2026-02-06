use super::*;

pub mod tree {
    use super::*;

    pub fn new(bitness: u8, initial_item: Expr) -> Expr {
        apply(var(format!("Tr{bitness}")), [initial_item])
    }

    pub fn node(left: Expr, right: Expr) -> Expr {
        pair::new(left, right)
    }

    pub fn get_left(array: Expr) -> Expr {
        pair::get_first(array)
    }

    pub fn get_right(array: Expr) -> Expr {
        pair::get_second(array)
    }

    /// The index bitness must match the tree bitness.
    pub fn index(array: Expr, index: Expr) -> Expr {
        apply(index, [array])
    }

    /// The index bitness must match the tree bitness.
    pub fn insert(bitness: u8, array: Expr, index: Expr, value: Expr) -> Expr {
        debug_assert!(bitness == 32 || bitness == 16);
        apply(
            var(format!("TIns{bitness}")),
            [array, number::to_bit_list_be(bitness, index), value],
        )
    }

    // TODO Is this useful? The idea is to generate a bit list at compile time if possible
    // pub fn static_insert(bitness: u8, array: Expr, index: u32, value: Expr) -> Expr {
    //     debug_assert!(bitness == 32 || bitness == 16);
    //     insert(bitness, array, todo!(), value)
    // }

    /// The index bitness must match the tree bitness.
    pub fn from(bitness: u8, items: impl IntoIterator<Item = Expr>, default_item: Expr) -> Expr {
        debug_assert!(bitness == 32 || bitness == 16);

        let mut items = items.into_iter().collect::<Vec<Expr>>();
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
            abs(
                ["insert", "array", "index", "value"],
                cond(
                    chain::get_head(var("index")),
                    tree::node(
                        tree::get_left(var("array")),
                        apply(
                            var("insert"),
                            [
                                tree::get_right(var("array")),
                                chain::get_tail(var("index")),
                                var("value"),
                            ],
                        ),
                    ),
                    tree::node(
                        apply(
                            var("insert"),
                            [
                                tree::get_left(var("array")),
                                chain::get_tail(var("index")),
                                var("value"),
                            ],
                        ),
                        tree::get_right(var("array")),
                    ),
                ),
            ),
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

    pub fn new(initial_item: Expr) -> Expr {
        tree::new(BITNESS, initial_item)
    }

    pub fn index(memory: Expr, address: Expr) -> Expr {
        tree::index(memory, address)
    }

    pub fn insert(memory: Expr, address: Expr, value: Expr) -> Expr {
        tree::insert(BITNESS, memory, address, value)
    }
}

pub mod table {
    use super::*;

    const BITNESS: u8 = 16;

    pub fn new() -> Expr {
        tree::new(BITNESS, unreachable())
    }

    pub fn from(items: impl IntoIterator<Item = Expr>) -> Expr {
        tree::from(BITNESS, items, unreachable())
    }

    pub fn index(table: Expr, address: Expr) -> Expr {
        tree::index(table, address)
    }

    pub fn insert(table: Expr, address: Expr, value: Expr) -> Expr {
        tree::insert(BITNESS, table, address, value)
    }
}

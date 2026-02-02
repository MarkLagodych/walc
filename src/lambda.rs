#[derive(Debug, Clone)]
pub enum Lambda {
    Variable {
        name: String,
    },
    Abstraction {
        variable: String,
        body: Box<Lambda>,
    },
    Application {
        function: Box<Lambda>,
        argument: Box<Lambda>,
    },
}

impl std::fmt::Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lambda::Variable { name } => {
                write!(f, "{}", name)
            }
            Lambda::Abstraction { variable, body } => {
                write!(f, "[{} {}]", variable, body)
            }
            Lambda::Application { function, argument } => {
                write!(f, "({} {})", function, argument)
            }
        }
    }
}

pub fn var(name: &str) -> Lambda {
    Lambda::Variable {
        name: name.to_string(),
    }
}

pub fn abs(vars: &[&str], body: Lambda) -> Lambda {
    let mut result = body;
    for var in vars.iter().rev() {
        result = Lambda::Abstraction {
            variable: var.to_string(),
            body: Box::new(result),
        };
    }
    result
}

pub fn apply(func: Lambda, args: impl Iterator<Item = Lambda>) -> Lambda {
    let mut result = func;
    for arg in args {
        result = Lambda::Application {
            function: Box::new(result),
            argument: Box::new(arg),
        };
    }
    result
}

pub fn def<'a, Defs>(definitions: Defs, body: Lambda) -> Lambda
where
    Defs: DoubleEndedIterator<Item = (&'a str, Lambda)>,
{
    let mut result = body;
    for (var, value) in definitions.rev() {
        result = apply(abs(&[var], result), [value].into_iter());
    }
    result
}

pub fn cond(condition: Lambda, then_branch: Lambda, else_branch: Lambda) -> Lambda {
    apply(condition, [else_branch, then_branch].into_iter())
}

pub fn unreachable() -> Lambda {
    if cfg!(feature = "debug-unreachable") {
        var("__UNREACHABLE__")
    } else {
        abs(&["U"], var("U"))
    }
}

pub fn rec(func: Lambda) -> Lambda {
    apply(func.clone(), [func].into_iter())
}

pub fn bit(b: bool) -> Lambda {
    if b { var("1") } else { var("0") }
}

pub mod pair {
    use super::*;

    pub fn new(first: Lambda, second: Lambda) -> Lambda {
        abs(&["P"], apply(var("P"), [first, second].into_iter()))
    }

    pub fn get_first(pair: Lambda) -> Lambda {
        apply(pair, [var("0")].into_iter())
    }

    pub fn get_second(pair: Lambda) -> Lambda {
        apply(pair, [var("1")].into_iter())
    }
}

pub mod either {
    use super::*;

    pub fn first(value: Lambda) -> Lambda {
        pair::new(var("0"), value)
    }

    pub fn second(value: Lambda) -> Lambda {
        pair::new(var("1"), value)
    }

    pub fn is_second(either: Lambda) -> Lambda {
        pair::get_first(either)
    }

    pub fn unwrap(either: Lambda) -> Lambda {
        pair::get_second(either)
    }
}

pub mod optional {
    use super::*;

    pub fn none() -> Lambda {
        either::first(unreachable())
    }

    pub fn some(value: Lambda) -> Lambda {
        either::second(value)
    }

    pub fn is_some(optional: Lambda) -> Lambda {
        either::is_second(optional)
    }

    pub fn unwrap(optional: Lambda) -> Lambda {
        either::unwrap(optional)
    }
}

pub mod number {
    use super::*;

    fn bytes_to_hex_string(be_bytes: &[u8]) -> String {
        String::from("0x")
            + &be_bytes
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<String>>()
                .join("")
    }

    fn number_const(be_bytes: &[u8]) -> Lambda {
        let var_name = bytes_to_hex_string(be_bytes);

        let mut result = var(&var_name);
        for byte in be_bytes {
            for i in (0..=7).rev() {
                let bit = bit((byte >> i) & 1 != 0);
                result = apply(result, [bit].into_iter());
            }
        }

        abs(&[&var_name], result)
    }

    pub fn u8_const(n: u8) -> Lambda {
        number_const(&n.to_be_bytes())
    }

    pub fn u32_const(n: u32) -> Lambda {
        number_const(&n.to_be_bytes())
    }

    pub fn u64_const(n: u64) -> Lambda {
        number_const(&n.to_be_bytes())
    }

    pub fn f32_const(n: f32) -> Lambda {
        number_const(&n.to_be_bytes())
    }

    pub fn f64_const(n: f64) -> Lambda {
        number_const(&n.to_be_bytes())
    }

    pub fn to_bit_list_be32(number: Lambda) -> Lambda {
        // I debugged this for two weeks :X
        // Yes, you really call a number with a function, not the other way around.
        apply(number, [var("ToBitsBE32")].into_iter())
    }
}

pub mod dyn_list {
    use super::*;

    pub fn empty() -> Lambda {
        optional::none()
    }

    pub fn node(head: Lambda, tail: Lambda) -> Lambda {
        optional::some(pair::new(head, tail))
    }

    pub fn is_not_empty(list: Lambda) -> Lambda {
        optional::is_some(list)
    }

    pub fn get_head(list: Lambda) -> Lambda {
        pair::get_first(optional::unwrap(list))
    }

    pub fn get_tail(list: Lambda) -> Lambda {
        pair::get_second(optional::unwrap(list))
    }
}

pub mod list {
    use super::*;

    pub fn empty() -> Lambda {
        unreachable()
    }

    pub fn node(head: Lambda, tail: Lambda) -> Lambda {
        pair::new(head, tail)
    }

    pub fn from(items: impl DoubleEndedIterator<Item = Lambda>) -> Lambda {
        let mut result = empty();
        for item in items.rev() {
            result = node(item, result);
        }
        result
    }

    pub fn get_head(list: Lambda) -> Lambda {
        pair::get_first(list)
    }

    pub fn get_tail(list: Lambda) -> Lambda {
        pair::get_second(list)
    }
}

pub mod tree_list {
    use super::*;

    pub fn default() -> Lambda {
        var("Arr")
    }

    pub fn node(left: Lambda, right: Lambda) -> Lambda {
        pair::new(left, right)
    }

    pub fn get_left(array: Lambda) -> Lambda {
        pair::get_first(array)
    }

    pub fn get_right(array: Lambda) -> Lambda {
        pair::get_second(array)
    }

    pub fn index(array: Lambda, index: Lambda) -> Lambda {
        apply(index, [array].into_iter())
    }

    pub fn insert(array: Lambda, index: Lambda, value: Lambda) -> Lambda {
        apply(var("ArrInsert"), [array, index, value].into_iter())
    }
}

pub mod walc_io {
    use super::*;

    pub fn end() -> Lambda {
        var("End")
    }

    pub fn output(out_byte: Lambda, next: Lambda) -> Lambda {
        apply(var("Out"), [out_byte, next].into_iter())
    }

    pub fn input(root_input_handler: Lambda) -> Lambda {
        apply(var("In"), [root_input_handler].into_iter())
    }
}

pub fn define_prelude(mut root: Lambda) -> Lambda {
    fn apply1(func: Lambda, arg: Lambda) -> Lambda {
        apply(func, [arg].into_iter())
    }

    root = def(
        [
            ("0", abs(&["x0", "x1"], var("x0"))),
            ("1", abs(&["x0", "x1"], var("x1"))),
            ////////////////////////////////////////////////////////////////////////////
            (
                "Out",
                abs(
                    &["out_byte", "next"],
                    optional::some(either::first(pair::new(var("out_byte"), var("next")))),
                ),
            ),
            (
                "In",
                abs(
                    &["input_handler"],
                    optional::some(either::second(var("input_handler"))),
                ),
            ),
            ("End", optional::none()),
            ////////////////////////////////////////////////////////////////////////////
            (
                "ToBitsBE32",
                abs(
                    &[
                        "b31", "b30", "b29", "b28", "b27", "b26", "b25", "b24", "b23", "b22",
                        "b21", "b20", "b19", "b18", "b17", "b16", "b15", "b14", "b13", "b12",
                        "b11", "b10", "b9", "b8", "b7", "b6", "b5", "b4", "b3", "b2", "b1", "b0",
                    ],
                    list::from(
                        [
                            var("b31"),
                            var("b30"),
                            var("b29"),
                            var("b28"),
                            var("b27"),
                            var("b26"),
                            var("b25"),
                            var("b24"),
                            var("b23"),
                            var("b22"),
                            var("b21"),
                            var("b20"),
                            var("b19"),
                            var("b18"),
                            var("b17"),
                            var("b16"),
                            var("b15"),
                            var("b14"),
                            var("b13"),
                            var("b12"),
                            var("b11"),
                            var("b10"),
                            var("b9"),
                            var("b8"),
                            var("b7"),
                            var("b6"),
                            var("b5"),
                            var("b4"),
                            var("b3"),
                            var("b2"),
                            var("b1"),
                            var("b0"),
                        ]
                        .into_iter(),
                    ),
                ),
            ),
            /////////////////////////////////////////////////////////////////////////////
            ("DefItem", number::u8_const(0)),
            // Indexable by 1 bit
            ("Arr1", tree_list::node(var("DefItem"), var("DefItem"))),
            ("Arr2", tree_list::node(var("Arr1"), var("Arr1"))),
            ("Arr3", tree_list::node(var("Arr2"), var("Arr2"))),
            ("Arr4", tree_list::node(var("Arr3"), var("Arr3"))),
            ("Arr5", tree_list::node(var("Arr4"), var("Arr4"))),
            ("Arr6", tree_list::node(var("Arr5"), var("Arr5"))),
            ("Arr7", tree_list::node(var("Arr6"), var("Arr6"))),
            ("Arr8", tree_list::node(var("Arr7"), var("Arr7"))),
            ("Arr9", tree_list::node(var("Arr8"), var("Arr8"))),
            ("Arr10", tree_list::node(var("Arr9"), var("Arr9"))),
            ("Arr11", tree_list::node(var("Arr10"), var("Arr10"))),
            ("Arr12", tree_list::node(var("Arr11"), var("Arr11"))),
            ("Arr13", tree_list::node(var("Arr12"), var("Arr12"))),
            ("Arr14", tree_list::node(var("Arr13"), var("Arr13"))),
            ("Arr15", tree_list::node(var("Arr14"), var("Arr14"))),
            ("Arr16", tree_list::node(var("Arr15"), var("Arr15"))),
            ("Arr17", tree_list::node(var("Arr16"), var("Arr16"))),
            ("Arr18", tree_list::node(var("Arr17"), var("Arr17"))),
            ("Arr19", tree_list::node(var("Arr18"), var("Arr18"))),
            ("Arr20", tree_list::node(var("Arr19"), var("Arr19"))),
            ("Arr21", tree_list::node(var("Arr20"), var("Arr20"))),
            ("Arr22", tree_list::node(var("Arr21"), var("Arr21"))),
            ("Arr23", tree_list::node(var("Arr22"), var("Arr22"))),
            ("Arr24", tree_list::node(var("Arr23"), var("Arr23"))),
            ("Arr25", tree_list::node(var("Arr24"), var("Arr24"))),
            ("Arr26", tree_list::node(var("Arr25"), var("Arr25"))),
            ("Arr27", tree_list::node(var("Arr26"), var("Arr26"))),
            ("Arr28", tree_list::node(var("Arr27"), var("Arr27"))),
            ("Arr29", tree_list::node(var("Arr28"), var("Arr28"))),
            ("Arr30", tree_list::node(var("Arr29"), var("Arr29"))),
            ("Arr31", tree_list::node(var("Arr30"), var("Arr30"))),
            // Indexable by 32 bits
            ("Arr", tree_list::node(var("Arr31"), var("Arr31"))),
            /////////////////////////////////////////////////////////////////////////////
            (
                "ArrInsert_",
                abs(
                    &["insert", "array", "index", "value"],
                    cond(
                        list::get_head(var("index")),
                        tree_list::node(
                            tree_list::get_left(var("array")),
                            apply(
                                var("insert"),
                                [
                                    tree_list::get_right(var("array")),
                                    list::get_tail(var("index")),
                                    var("value"),
                                ]
                                .into_iter(),
                            ),
                        ),
                        tree_list::node(
                            apply(
                                var("insert"),
                                [
                                    tree_list::get_left(var("array")),
                                    list::get_tail(var("index")),
                                    var("value"),
                                ]
                                .into_iter(),
                            ),
                            tree_list::get_right(var("array")),
                        ),
                    ),
                ),
            ),
            // Consumes 0 bits of index
            (
                "ArrInsert0",
                abs(&["array", "index", "value"], var("value")),
            ),
            // Consumes 1 bit of index
            ("ArrInsert1", apply1(var("ArrInsert_"), var("ArrInsert0"))),
            ("ArrInsert2", apply1(var("ArrInsert_"), var("ArrInsert1"))),
            ("ArrInsert3", apply1(var("ArrInsert_"), var("ArrInsert2"))),
            ("ArrInsert4", apply1(var("ArrInsert_"), var("ArrInsert3"))),
            ("ArrInsert5", apply1(var("ArrInsert_"), var("ArrInsert4"))),
            ("ArrInsert6", apply1(var("ArrInsert_"), var("ArrInsert5"))),
            ("ArrInsert7", apply1(var("ArrInsert_"), var("ArrInsert6"))),
            ("ArrInsert8", apply1(var("ArrInsert_"), var("ArrInsert7"))),
            ("ArrInsert9", apply1(var("ArrInsert_"), var("ArrInsert8"))),
            ("ArrInsert10", apply1(var("ArrInsert_"), var("ArrInsert9"))),
            ("ArrInsert11", apply1(var("ArrInsert_"), var("ArrInsert10"))),
            ("ArrInsert12", apply1(var("ArrInsert_"), var("ArrInsert11"))),
            ("ArrInsert13", apply1(var("ArrInsert_"), var("ArrInsert12"))),
            ("ArrInsert14", apply1(var("ArrInsert_"), var("ArrInsert13"))),
            ("ArrInsert15", apply1(var("ArrInsert_"), var("ArrInsert14"))),
            ("ArrInsert16", apply1(var("ArrInsert_"), var("ArrInsert15"))),
            ("ArrInsert17", apply1(var("ArrInsert_"), var("ArrInsert16"))),
            ("ArrInsert18", apply1(var("ArrInsert_"), var("ArrInsert17"))),
            ("ArrInsert19", apply1(var("ArrInsert_"), var("ArrInsert18"))),
            ("ArrInsert20", apply1(var("ArrInsert_"), var("ArrInsert19"))),
            ("ArrInsert21", apply1(var("ArrInsert_"), var("ArrInsert20"))),
            ("ArrInsert22", apply1(var("ArrInsert_"), var("ArrInsert21"))),
            ("ArrInsert23", apply1(var("ArrInsert_"), var("ArrInsert22"))),
            ("ArrInsert24", apply1(var("ArrInsert_"), var("ArrInsert23"))),
            ("ArrInsert25", apply1(var("ArrInsert_"), var("ArrInsert24"))),
            ("ArrInsert26", apply1(var("ArrInsert_"), var("ArrInsert25"))),
            ("ArrInsert27", apply1(var("ArrInsert_"), var("ArrInsert26"))),
            ("ArrInsert28", apply1(var("ArrInsert_"), var("ArrInsert27"))),
            ("ArrInsert29", apply1(var("ArrInsert_"), var("ArrInsert28"))),
            ("ArrInsert30", apply1(var("ArrInsert_"), var("ArrInsert29"))),
            ("ArrInsert31", apply1(var("ArrInsert_"), var("ArrInsert30"))),
            // Consumes 32 bits of index
            ("ArrInsert32", apply1(var("ArrInsert_"), var("ArrInsert31"))),
            (
                "ArrInsert",
                abs(
                    &["array", "index", "value"],
                    apply(
                        var("ArrInsert32"),
                        [
                            var("array"),
                            number::to_bit_list_be32(var("index")),
                            var("value"),
                        ]
                        .into_iter(),
                    ),
                ),
            ),
        ]
        .into_iter(),
        root,
    );

    root
}

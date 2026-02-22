use super::*;

use crate::analyzer::{Operator, ValType};

// An ordered set makes the resulting code slightly nicer: the constants are defined in order
use std::collections::BTreeSet as Set;

/// This struct accumulates all numeric constants throughout the WASM module in order to
/// reduce the resulting code size.
#[derive(Default)]
pub struct ConstantDefinitionBuilder {
    bytes: Set<u8>,
    ids: Set<u16>,
    i32s: Set<u32>,
    i64s: Set<u64>,
}

pub type Byte = Expr;
pub type Id = Expr;
pub type I32 = Expr;
pub type I64 = Expr;
/// Any of: Id, I32, I64
pub type Number = Expr;

impl ConstantDefinitionBuilder {
    #[allow(unused)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> DefinitionBuilder {
        let mut b = DefinitionBuilder::new();

        for &n in &self.bytes {
            if n == 0 {
                continue; // "0b" is pre-defined in the prelude
            }
            b.def(format!("{:X}b", n), byte_expr(n));
        }
        for &n in &self.ids {
            b.def(format!("{:X}n", n), id_expr(n));
        }
        for &n in &self.i32s {
            b.def(format!("{:X}i", n), i32_expr(n));
        }
        for &n in &self.i64s {
            b.def(format!("{:X}l", n), i64_expr(n));
        }

        b
    }

    pub fn byte_const(&mut self, byte: u8) -> Byte {
        if byte == 0 {
            return null_byte();
        }

        self.bytes.insert(byte);
        var(format!("{:X}b", byte))
    }

    pub fn id_const(&mut self, id: u16) -> Id {
        self.ids.insert(id);
        self.bytes.extend(id.to_be_bytes());
        var(format!("{:X}n", id))
    }

    pub fn i32_const(&mut self, n: u32) -> I32 {
        self.i32s.insert(n);
        self.bytes.extend(n.to_be_bytes());
        var(format!("{:X}i", n))
    }

    pub fn i64_const(&mut self, n: u64) -> I64 {
        self.i64s.insert(n);
        self.bytes.extend(n.to_be_bytes());
        var(format!("{:X}l", n))
    }

    pub fn with_init_value(&mut self, init_value: &Operator) -> Number {
        match init_value {
            Operator::I32Const { value } => self.i32_const(*value as u32),
            Operator::I64Const { value } => self.i64_const(*value as u64),
            Operator::F32Const { value } => self.i32_const(value.bits()),
            Operator::F64Const { value } => self.i64_const(value.bits()),
            _ => unreachable!("non-const initializers are unsupported"),
        }
    }

    pub fn default_const(&mut self, ty: ValType) -> Number {
        match ty {
            ValType::I32 => self.i32_const(0),
            ValType::I64 => self.i64_const(0),
            ValType::F32 => self.i32_const(0),
            ValType::F64 => self.i64_const(0),
            _ => unreachable!("non-const initializers are unsupported"),
        }
    }
}

pub fn define_prelude(b: &mut DefinitionBuilder) {
    b.def(
        "Byte",
        abs(
            (0..8).rev().map(|i| format!("{i}")),
            list::from((0..8).rev().map(|i| var(format!("{i}")))),
        ),
    );

    // Reverses the bits of "x" and places them on top of "tail"
    b.def(
        "RevEx",
        abs(["rev", "x", "tail"], {
            select(
                list::is_not_empty(var("x")),
                var("tail"),
                apply(
                    rec(var("rev")),
                    [
                        list::get_tail(var("x")),
                        list::node(list::get_head(var("x")), var("tail")),
                    ],
                ),
            )
        }),
    );

    b.def(
        "Rev",
        abs(["x"], apply(rec(var("RevEx")), [var("x"), list::empty()])),
    );

    for (name, byte_count) in [("Id", 2), ("Int", 4), ("Long", 8)] {
        b.def(
            name,
            abs(
                (0..byte_count).rev().map(|i| format!("b{i}")),
                join_bytes((0..byte_count).rev().map(|i| var(format!("b{i}")))),
            ),
        );
    }

    b.def("0b", byte_expr(0));
}

fn join_bytes(be_bytes: impl IntoIterator<Item = Expr>) -> Expr {
    let mut expr = list::empty();
    for byte in be_bytes.into_iter() {
        expr = apply(rec(var("RevEx")), [byte, expr]);
    }

    expr
}

pub fn reverse_bits(expr: Number) -> Number {
    apply(var("Rev"), [expr])
}

fn byte_expr(byte: u8) -> Byte {
    let ith_bit = |i: u8| bit((byte >> i) & 1 != 0);

    apply(var("Byte"), (0..8).rev().map(ith_bit))
}

fn id_expr(id: u16) -> Id {
    apply(
        var("Id"),
        id.to_be_bytes().iter().map(|&b| var(format!("{b:X}b"))),
    )
}

pub fn make_i32(bytes: impl IntoIterator<Item = Expr>) -> I32 {
    apply(var("Int"), bytes)
}

fn i32_expr(n: u32) -> I32 {
    make_i32(n.to_be_bytes().iter().map(|&b| var(format!("{b:X}b"))))
}

pub fn make_i64(bytes: impl IntoIterator<Item = Expr>) -> I64 {
    apply(var("Long"), bytes)
}

fn i64_expr(n: u64) -> I64 {
    make_i64(n.to_be_bytes().iter().map(|&b| var(format!("{b:X}b"))))
}

pub fn null_byte() -> Byte {
    var("0b")
}

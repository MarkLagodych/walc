//! Numbers are represented as simple tuples of bits:
//! `[getter ((...((getter bit<N>) bit<N-1>)... bit1) bit0)]`.
//! Thus when you want to get actual bits from a number, you must *apply a function to a number*,
//! and not vice versa!

use super::*;

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
/// Any of: Byte, Id, I32, I64
pub type Number = Expr;

impl ConstantDefinitionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self, b: &mut DefinitionBuilder) {
        for &n in &self.bytes {
            b.def(format!("{:02x}", n), Self::byte_expr(n));
        }
        for &n in &self.ids {
            b.def(format!("{:04x}", n), Self::number_expr(&n.to_be_bytes()));
        }
        for &n in &self.i32s {
            b.def(format!("{:08x}", n), Self::number_expr(&n.to_be_bytes()));
        }
        for &n in &self.i64s {
            b.def(format!("{:016x}", n), Self::number_expr(&n.to_be_bytes()));
        }
    }

    fn byte_expr(byte: u8) -> Byte {
        let ith_bit = |i: u8| -> Bit { bit((byte >> i) & 1 != 0) };

        abs(["x"], apply(var("x"), (0..8).rev().map(ith_bit)))
    }

    fn number_expr(be_bytes: &[u8]) -> Number {
        let mut expr = var("n");
        for &byte in be_bytes {
            expr = apply(var(format!("{:02x}", byte)), [expr]);
        }
        abs(["n"], expr)
    }

    pub fn byte_const(&mut self, byte: u8) -> Byte {
        self.bytes.insert(byte);
        var(format!("{:02x}", byte))
    }

    pub fn id_const(&mut self, id: u16) -> Id {
        self.ids.insert(id);
        self.bytes.extend(id.to_be_bytes());
        var(format!("{:04x}", id))
    }

    pub fn i32_const(&mut self, n: u32) -> I32 {
        self.i32s.insert(n);
        self.bytes.extend(n.to_be_bytes());
        var(format!("{:08x}", n))
    }

    pub fn i64_const(&mut self, n: u64) -> I64 {
        self.i64s.insert(n);
        self.bytes.extend(n.to_be_bytes());
        var(format!("{:016x}", n))
    }
}

pub fn to_bit_list_be(bitness: u8, number: Number) -> unsafe_list::UnsafeList {
    debug_assert!(bitness == 32 || bitness == 16);

    apply(number, [var(format!("ToBitsBE{bitness}"))])
}

pub fn define_prelude(b: &mut DefinitionBuilder) {
    for bitness in [16, 32] {
        b.def(
            format!("ToBitsBE{bitness}"),
            abs(
                (0..bitness).rev().map(|i| i.to_string()),
                unsafe_list::from((0..bitness).rev().map(|i| var(i.to_string()))),
            ),
        );
    }
}

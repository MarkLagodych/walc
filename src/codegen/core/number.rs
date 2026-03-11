//! Numbers are represented as lists of bits:
//!
//! * `I32`/`I64` are **little-endian** lists of 32/64 bits.
//!   These numbers are frequently used in addition and multiplication, which are performed
//!   in **little endian**, so it's better to have these numbers in LE to avoid unnecessary
//!   conversions.
//!
//! * `Byte`s are **big-endian** lists of 8 bits.
//!   Bytes are used to store data in the memory and in code.
//!
//!   All wider numbers are constructed from bytes by joining them.
//!   This is necessary when loading from the memory and also makes the code for numeric constants
//!   smaller.
//!   However, any process of converting between a little-endian number and its parts (e.g. bytes)
//!   reverses the bits (see the implementation of this module or
//!   [`crate::codegen::util::UtilGenerator::num_chop`]),
//!   thus it is more efficient to store bytes as **big-endian**.
//!
//! * `Id`s are **big-endian** lists of 16 bits.
//!   The only operation performed on these is table indexing, which is done in **big endian**.
//!
//!   Also, the conversion from little-endian `I32` to `Id` (used in the `call_indirect`
//!   instruction) directly gives a big-endian number
//!   (see [`crate::codegen::util::UtilGenerator::num_chop`]).
//!
//! # Construction of numbers
//!
//! ## Bytes
//!
//! Bytes are stored directly as lists of bits.
//!
//! As a single point of reference, bytes are always considered to be big-endian.
//!
//! For example, the byte `0x0f` (written in BE, 15 in decimal) is directly stored as the list
//! `(cons 0 (cons 0 (cons 0 (cons 0 (cons 1 (cons 1 (cons 1 (cons 1 empty))))))))`.
//!
//! ## I32/I64
//!
//! These numbers are little-endian and are constructed from big-endian lists of big-endian bytes
//! (the endianness is reversed when converting from and to bytes).
//!
//! For example, a number `0x87654321` (written in BE) is stored as `(I32 0x87 0x65 0x43 0x21)`,
//! where `I32` is a function that constructs an `I32` out of 4 bytes.
//!
//! # Ids
//!
//! These numbers are big-endian and are constructed from little-endian lists of reversed bytes.
//!
//! For example, an ID `0x4321` (written in BE) is stored as `(Id 0x84 0xC2)`,
//! where `Id` is a function that constructs an ID out of bytes,
//! `0x84` is reversed `0x21` and `0xC2` is reversed `0x43`.
//!
//! Even though this is unintuitive and produces unreadable lambda-expression code, this is
//! more efficient.

use super::*;

use crate::analyzer::{Operator, ValType};

// An ordered set makes the resulting code slightly nicer: the constants are defined in order
use std::collections::BTreeSet as Set;

/// This struct accumulates all numeric constants throughout the WASM module in order to
/// define each only once and thus reduce the resulting code size.
///
/// To further reduce the code size, every integer wider than a byte is constructed from whole bytes
/// and not individual bits.
#[derive(Default)]
pub struct NumberGenerator {
    bytes: Set<u8>,
    ids: Set<u16>,
    i32s: Set<u32>,
    i64s: Set<u64>,
}

pub type Byte = Expr;
pub type Id = Expr;
pub type I32 = Expr;
pub type I64 = Expr;
/// Any of: I32, I64
pub type Number = Expr;

impl NumberGenerator {
    #[allow(unused)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate(self, b: &mut LetExprBuilder) {
        for &n in &self.bytes {
            if n == 0 {
                continue; // "0b" is pre-defined in the prelude
            }
            b.def(format!("{:X}b", n), byte_expr(n));
        }
        for &n in &self.ids {
            b.def(format!("{:X}x", n), id_expr(n));
        }
        for &n in &self.i32s {
            b.def(format!("{:X}i", n), i32_expr(n));
        }
        for &n in &self.i64s {
            b.def(format!("{:X}I", n), i64_expr(n));
        }
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

        self.bytes
            .extend(id.to_le_bytes().map(|byte| byte.reverse_bits()));

        var(format!("{:X}x", id))
    }

    pub fn i32_const(&mut self, n: u32) -> I32 {
        self.i32s.insert(n);
        self.bytes.extend(n.to_be_bytes());
        var(format!("{:X}i", n))
    }

    pub fn i64_const(&mut self, n: u64) -> I64 {
        self.i64s.insert(n);
        self.bytes.extend(n.to_be_bytes());
        var(format!("{:X}I", n))
    }

    pub fn with_init_value(&mut self, init_value: &Operator) -> Number {
        match init_value {
            Operator::I32Const { value } => self.i32_const(*value as u32),
            Operator::I64Const { value } => self.i64_const(*value as u64),
            // TODO floats
            // Should floats be handled the same way as integers?
            // They can be represented in a different way, e.g. as a triple of (sign, exp, mantissa)
            Operator::F32Const { value } => self.i32_const(value.bits()),
            Operator::F64Const { value } => self.i64_const(value.bits()),
            // FIXME allow global.get for data offsets
            _ => unreachable!("non-const initializers are unsupported"),
        }
    }

    pub fn default_const(&mut self, ty: ValType) -> Number {
        match ty {
            ValType::I32 => self.i32_const(0),
            ValType::I64 => self.i64_const(0),
            // TODO floats
            ValType::F32 => self.i32_const(0),
            ValType::F64 => self.i64_const(0),
            _ => unreachable!(),
        }
    }
}

/// Joins a sequence of bytes into a single list of bits with reversed endianness.
///
/// If the input is a big-endian list of bytes, the output is little-endian, and vice versa.
fn join_reversed(bytes: impl IntoIterator<Item = Expr>) -> Expr {
    let mut expr = list::empty();
    for byte in bytes.into_iter() {
        expr = apply(rec(var("JoinRev")), [expr, byte]);
    }

    expr
}

pub fn generate_defs(b: &mut LetExprBuilder) {
    // The arguments are big-endian, and the list is big-endian
    b.def(
        "Byte",
        abs(
            (0..8).rev().map(|i| format!("{i}")),
            list::from((0..8).rev().map(|i| var(format!("{i}")))),
        ),
    );

    // Reverses the bits of "x" and places them on top of "tail"
    b.def_rec(
        "JoinRev",
        abs(["tail", "x"], {
            select(
                list::is_not_empty(var("x")),
                var("tail"),
                apply(
                    rec(var("JoinRev")),
                    [
                        list::node(list::get_head(var("x")), var("tail")),
                        list::get_tail(var("x")),
                    ],
                ),
            )
        }),
    );

    b.def("Rev", apply(rec(var("JoinRev")), [list::empty()]));

    // I32/I64 expect a big-endian sequence of bytes as arguments
    // Id expects a little-endian sequence of reversed bytes as arguments
    for (name, byte_count) in [("Id", 2), ("I32", 4), ("I64", 8)] {
        b.def(
            name,
            abs(
                (0..byte_count).map(|i| format!("b{i}")),
                join_reversed((0..byte_count).map(|i| var(format!("b{i}")))),
            ),
        );
    }

    // Trees depend on this as the default value for uninitialized elements.
    b.def("0b", byte_expr(0));
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
        id.to_le_bytes()
            .iter()
            .map(|byte| byte.reverse_bits())
            .map(|b| var(format!("{b:X}b"))),
    )
}

/// Makes an I32 from big-endian sequence of 4 bytes.
pub fn make_i32(be_bytes: impl IntoIterator<Item = Expr>) -> I32 {
    apply(var("I32"), be_bytes)
}

fn i32_expr(n: u32) -> I32 {
    make_i32(n.to_be_bytes().iter().map(|&b| var(format!("{b:X}b"))))
}

/// Makes an I64 from big-endian sequence of 8 bytes.
pub fn make_i64(be_bytes: impl IntoIterator<Item = Expr>) -> I64 {
    apply(var("I64"), be_bytes)
}

fn i64_expr(n: u64) -> I64 {
    make_i64(n.to_be_bytes().iter().map(|&b| var(format!("{b:X}b"))))
}

pub fn null_byte() -> Byte {
    var("0b")
}

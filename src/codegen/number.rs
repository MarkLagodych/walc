//! Numbers are represented as simple tuples of bits:
//! `[getter ((...((getter bit<N>) bit<N-1>)... bit1) bit0)]`.
//! Thus when you want to get actual bits from a number, you must *apply a function to a number*,
//! and not vice versa!

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
/// Any of: Byte, Id, I32, I64
pub type Number = Expr;

fn byte_expr(byte: u8) -> Byte {
    let ith_bit = |i: u8| bit((byte >> i) & 1 != 0);

    abs(["x"], apply(var("x"), (0..8).rev().map(ith_bit)))
}

fn number_expr(be_bytes: &[u8]) -> Number {
    let mut expr = var("n");
    for &byte in be_bytes {
        expr = apply(var(format!("{:02X}", byte)), [expr]);
    }
    abs(["n"], expr)
}

impl ConstantDefinitionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self, b: &mut DefinitionBuilder) {
        for &n in &self.bytes {
            if n == 0 {
                continue; // "00" is pre-defined in the prelude
            }
            b.def(format!("{:02X}", n), byte_expr(n));
        }
        for &n in &self.ids {
            b.def(format!("{:04X}", n), number_expr(&n.to_be_bytes()));
        }
        for &n in &self.i32s {
            b.def(format!("{:08X}", n), number_expr(&n.to_be_bytes()));
        }
        for &n in &self.i64s {
            b.def(format!("{:016X}", n), number_expr(&n.to_be_bytes()));
        }
    }

    pub fn byte_const(&mut self, byte: u8) -> Byte {
        if byte == 0 {
            return var("0_8");
        }

        self.bytes.insert(byte);
        var(format!("{:02X}", byte))
    }

    pub fn id_const(&mut self, id: u16) -> Id {
        // The ID of 0 is not special and does not need to be optimized

        self.ids.insert(id);
        self.bytes.extend(id.to_be_bytes());
        var(format!("{:04X}", id))
    }

    pub fn i32_const(&mut self, n: u32) -> I32 {
        if n == 0 {
            return var("0_32");
        }

        self.i32s.insert(n);
        self.bytes.extend(n.to_be_bytes());
        var(format!("{:08X}", n))
    }

    pub fn i64_const(&mut self, n: u64) -> I64 {
        if n == 0 {
            return var("0_64");
        }

        self.i64s.insert(n);
        self.bytes.extend(n.to_be_bytes());
        var(format!("{:016X}", n))
    }

    pub fn with_init_value(&mut self, init_value: &Operator) -> Number {
        match init_value {
            Operator::I32Const { value } => self.i32_const(*value as u32),
            Operator::I64Const { value } => self.i64_const(*value as u64),
            Operator::F32Const { value } => self.i32_const(value.bits()),
            Operator::F64Const { value } => self.i64_const(value.bits()),
            _ => unreachable!("WASM 1.0 cannot have const initializers other than I32/I64/F32/F64"),
        }
    }

    pub fn default_const(&mut self, ty: ValType) -> Number {
        match ty {
            ValType::I32 => self.i32_const(0),
            ValType::I64 => self.i64_const(0),
            ValType::F32 => self.i32_const(0),
            ValType::F64 => self.i64_const(0),
            _ => unreachable!("WASM 1.0 cannot have value types other than I32/I64/F32/F64"),
        }
    }
}

pub fn define_prelude(b: &mut DefinitionBuilder) {
    b.def("00", byte_expr(0));
    b.def("0i", number_expr(&0u32.to_be_bytes())); // "int"
    b.def("0l", number_expr(&0u64.to_be_bytes())); // "long"
}

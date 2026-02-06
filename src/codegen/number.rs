use super::*;

// An ordered set makes the resulting code slightly nicer: the constants are defined in order
use std::collections::BTreeSet as Set;

/// This struct accumulates all numeric constants throughout the WASM module in order to
/// reduce the resulting code size.
#[derive(Default)]
pub struct ConstantStore {
    bytes: Set<u8>,
    ids: Set<u16>,
    i32s: Set<u32>,
    i64s: Set<u64>,
}

impl ConstantStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn define_constants(&self, b: &mut DefinitionBuilder) {
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

    fn byte_expr(byte: u8) -> Expr {
        let ith_bit = |i: u8| -> Expr { bit((byte >> i) & 1 != 0) };

        abs(["x"], apply(var("x"), (0..8).rev().map(ith_bit)))
    }

    fn number_expr(be_bytes: &[u8]) -> Expr {
        let mut expr = var("n");
        for &byte in be_bytes {
            expr = apply(var(format!("{:02x}", byte)), [expr]);
        }
        abs(["n"], expr)
    }

    pub fn byte_const(&mut self, byte: u8) -> Expr {
        self.bytes.insert(byte);
        var(format!("{:02x}", byte))
    }

    pub fn id_const(&mut self, id: u16) -> Expr {
        self.ids.insert(id);
        self.bytes.extend(id.to_be_bytes());
        var(format!("{:04x}", id))
    }

    pub fn i32_const(&mut self, n: u32) -> Expr {
        self.i32s.insert(n);
        self.bytes.extend(n.to_be_bytes());
        var(format!("{:08x}", n))
    }

    pub fn i64_const(&mut self, n: u64) -> Expr {
        self.i64s.insert(n);
        self.bytes.extend(n.to_be_bytes());
        var(format!("{:016x}", n))
    }
}

pub fn to_bit_list_be(bitness: u8, number: Expr) -> Expr {
    debug_assert!(bitness == 32 || bitness == 16);

    // I debugged this for two weeks O_o
    // Yes, you really call a number with a function, not the other way around.
    apply(number, [var(format!("ToBitsBE{bitness}"))])
}

pub(super) fn define_prelude(b: &mut DefinitionBuilder) {
    for bitness in [16, 32] {
        b.def(
            format!("ToBitsBE{bitness}"),
            abs(
                (0..bitness).rev().map(|i| i.to_string()),
                chain::from((0..bitness).rev().map(|i| var(i.to_string()))),
            ),
        );
    }
}

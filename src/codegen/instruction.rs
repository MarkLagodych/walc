use super::*;

use crate::analyzer::{Operator, ValType};

use std::collections::BTreeSet as Set;

pub type Instruction = Expr;

pub fn instruction(op: &Operator) -> Instruction {
    unreachable()
}

enum InstrId {
    I32Const,
    I64Const,
    // TODO more instructions
}

#[derive(Default)]
pub struct InstructionDefinitionBuilder {
    instr_set: Set<InstrId>,
}

impl InstructionDefinitionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self, b: &mut DefinitionBuilder) {}
}

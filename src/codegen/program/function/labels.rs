use crate::codegen::*;

use super::*;

#[derive(Default)]
pub struct LabelDefinitionBuilder<'a> {
    /// Defines label variables.
    defs: DefinitionBuilder,

    /// Stack of `end` labels (for `loop`, `if`, `block`).
    end_labels: Vec<InstructionChain>,

    /// Stack of `else` labels (only for `if` blocks).
    else_labels: Vec<Option<InstructionChain>>,

    /// Stack of operators that start blocks corresponding to `end` operators.
    /// When reading `End` operators, pop the corresponding block start from this stack
    block_start_ops: Vec<&'a Operator<'a>>,

    /// Stack of block starting operators that tracks the current block nesting structure.
    blocks: Vec<&'a Operator<'a>>,

    next_label_id: u32,
}

pub struct LabelInfo<'a> {
    /// Stack of block opening operators (`loop`, `if`, `block`).
    pub blocks: &'a [&'a Operator<'a>],

    /// Stack of `end` labels (for `loop`, `if`, `block`).
    pub end_labels: &'a [InstructionChain],

    /// `else` label corresponding to the current `if` block, if any.
    pub else_label: Option<InstructionChain>,
}

impl<'a> LabelDefinitionBuilder<'a> {
    pub fn new(ops: &'a [Operator<'a>]) -> Self {
        let mut me = Self::default();
        me.read_block_start_ops(ops);
        me
    }

    pub fn build(self, chain: InstructionChain) -> InstructionChain {
        self.defs.build(chain)
    }

    fn read_block_start_ops(&mut self, ops: &'a [Operator<'a>]) {
        let mut blocks = Vec::new();

        for op in ops.iter() {
            match op {
                Operator::Loop { .. } | Operator::Block { .. } | Operator::If { .. } => {
                    blocks.push(op);
                }
                Operator::End => {
                    self.block_start_ops.push(blocks.pop().unwrap());
                }
                _ => {}
            }
        }
    }

    fn insert_label(&mut self, chain: InstructionChain) -> InstructionChain {
        let label_name = format!("_{:x}", self.next_label_id);
        self.next_label_id += 1;

        self.defs.def(label_name.clone(), chain);

        var(label_name)
    }

    pub fn update_label_info(&mut self, op: &Operator) {
        match op {
            Operator::Loop { .. } | Operator::Block { .. } | Operator::If { .. } => {
                self.blocks.pop();
            }
            Operator::End => {
                self.blocks.push(self.block_start_ops.pop().unwrap());
            }
            _ => {}
        }
    }

    pub fn insert_label_if_needed(
        &mut self,
        mut chain: InstructionChain,
        op: &Operator,
    ) -> InstructionChain {
        match op {
            Operator::Loop { .. } | Operator::Block { .. } | Operator::If { .. } => {
                chain = self.insert_label(chain);

                self.end_labels.pop();

                self.else_labels.pop();

                chain
            }
            Operator::Else { .. } => {
                chain = self.insert_label(chain);

                self.else_labels.pop();
                self.else_labels.push(Some(chain.clone()));

                chain
            }
            Operator::End => {
                chain = self.insert_label(chain);

                self.end_labels.push(chain.clone());

                self.else_labels.push(None);

                chain
            }
            _ => chain,
        }
    }

    pub fn get_label_info(&self) -> LabelInfo<'_> {
        LabelInfo {
            blocks: &self.blocks,
            end_labels: &self.end_labels,
            else_label: self.else_labels.last().cloned().unwrap_or_default(),
        }
    }
}

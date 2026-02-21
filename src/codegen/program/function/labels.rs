use crate::codegen::*;

use super::*;

#[derive(Default)]
pub struct LabelDefinitionBuilder<'a> {
    /// Defines label variables.
    defs: DefinitionBuilder,

    /// Stack of `end` labels (for `loop`, `if`, `block`).
    end_labels: Vec<EndLabel<'a>>,

    /// Stack of `else` labels (only for `if` blocks).
    else_labels: Vec<ElseLabel>,

    /// Stack of operators that start blocks.
    /// When reading `End` operators, pop the corresponding block start from this stack
    block_start_ops: Vec<Option<&'a Operator<'a>>>,

    next_label_id: u32,
}

pub struct LabelInfo<'a> {
    pub end_labels: &'a [EndLabel<'a>],
    pub else_labels: &'a [ElseLabel],
}

pub struct EndLabel<'a> {
    /// The subchain that the label points to.
    subchain: InstructionChain,

    // If None, this label is the function body's end label.
    // Otherwise, this is the block opening operator corresponding to this label.
    block_start_op: Option<&'a Operator<'a>>,
}

pub struct ElseLabel {
    /// The subchain that the label points to.
    subchain: InstructionChain,
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
        let mut stack = Vec::new();

        for op in ops.iter() {
            match op {
                Operator::Loop { .. } | Operator::Block { .. } | Operator::If { .. } => {
                    stack.push(op);
                }
                Operator::End => {
                    self.block_start_ops.push(stack.pop());
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

    fn push_end_label(&mut self, chain: InstructionChain) {
        self.end_labels.push(EndLabel {
            subchain: chain,
            block_start_op: self.block_start_ops.pop().unwrap(),
        });
    }

    fn pop_end_label(&mut self) -> EndLabel<'a> {
        self.end_labels.pop().unwrap()
    }

    fn push_else_label(&mut self, chain: InstructionChain) {
        self.else_labels.push(ElseLabel { subchain: chain });
    }

    fn pop_else_label(&mut self) -> ElseLabel {
        self.else_labels.pop().unwrap()
    }

    pub fn insert_label_if_needed(
        &mut self,
        mut chain: InstructionChain,
        op: &Operator,
    ) -> InstructionChain {
        match op {
            Operator::Loop { .. } | Operator::Block { .. } | Operator::If { .. } => {
                chain = self.insert_label(chain);

                self.pop_end_label();

                if matches!(op, Operator::If { .. }) {
                    self.pop_else_label();
                }

                chain
            }
            Operator::Else { .. } => {
                chain = self.insert_label(chain);

                self.push_else_label(chain.clone());

                chain
            }
            Operator::End => {
                chain = self.insert_label(chain);

                self.push_end_label(chain.clone());

                chain
            }
            _ => chain,
        }
    }

    pub fn get_label_info(&self) -> LabelInfo<'_> {
        LabelInfo {
            end_labels: &self.end_labels,
            else_labels: &self.else_labels,
        }
    }
}

use super::*;

use crate::analyzer::*;

/// Made of instructions (see [`instruction::Instruction`]) similarly to (linked) lists
/// (see [`list::List`]), so e.g. `(Instr1 (Instr2 (Instr3 unreachable)))` is a typical chain of
/// simple (non-control) instructions.
///
/// However, instruction chains get more complicated when it comes to control instructions.
/// For them we need special "labels" that point to specific segments of the chain, so that
/// control instructions can "jump" to them.
///
/// This is done by constructing the chain in parts: each subchain is assigned to a variable
/// (whose name is practically a label) and is used as a tail for the next subchain.
///
/// For example, consider the following instructions:
/// ```wat
/// block
///     i32.eqz
///     br_if 0     ;; refers to label 1
///     call $foo
/// end             ;; label 1
/// call $bar
/// ;;(unreachable) ;; label 0
/// ```
/// Note that labels are indexed relatively and refer to block nesting depth rather than
/// concrete labels.
///
/// The corresponding instruction chain will look like this:
/// ```text
/// let label0 = unreachable in
/// let label1 = (end (call<bar> label0)) in
/// (block (i32_eqz (br_if<label1> (call<foo> label1))))
/// ```
///
/// The resulting `br_if` instruction will jump either to the next instruction (i.e. `call<foo>`)
/// or to `label1`.
pub type InstructionChain = Expr;

pub struct FunctionBuildInfo<'a> {
    pub func: &'a Func<'a>,
    pub types: &'a GlobalTypeInfo,
    pub consts: &'a mut number::ConstantDefinitionBuilder,
    pub instrs: &'a mut instruction::InstructionDefinitionBuilder,
}

pub fn function(info: FunctionBuildInfo) -> InstructionChain {
    let mut chain_builder = InstructionChainBuilder::new(info.func.operators);

    let mut chain: InstructionChain = chain_builder.start_chain(info.instrs.leave(info.func));

    // Skip the last "end" operator
    for op in info.func.operators.iter().rev().skip(1) {
        let instr = info
            .instrs
            .instruction(&mut instruction::InstructionBuildInfo {
                op,
                types: info.types,
                consts: info.consts,
                end_labels: &chain_builder.labels,
                else_labels: &chain_builder.else_labels,
            });

        chain = chain_builder.insert_instruction(chain, instr);
        chain = chain_builder.insert_label_if_needed(chain, op);
    }

    chain = chain_builder.insert_instruction(chain, info.instrs.enter(info.func, info.consts));

    chain_builder.defs.build(chain)
}

pub fn input_function(instrs: &mut instruction::InstructionDefinitionBuilder) -> InstructionChain {
    InstructionChainBuilder::single_instruction(instrs.input_and_return())
}

pub fn output_function(instrs: &mut instruction::InstructionDefinitionBuilder) -> InstructionChain {
    InstructionChainBuilder::single_instruction(instrs.output_and_return())
}

pub fn exit_function(instrs: &mut instruction::InstructionDefinitionBuilder) -> InstructionChain {
    InstructionChainBuilder::single_instruction(instrs.exit())
}

#[derive(Default)]
struct InstructionChainBuilder<'a> {
    /// Stack of `end` labels (for `loop`, `if`, `block`).
    labels: Vec<EndLabel<'a>>,

    /// Stack of `else` labels (only for `if` blocks).
    else_labels: Vec<ElseLabel>,

    /// Stack of operators that start blocks.
    /// When reading `End` operators, pop the corresponding block start from this stack
    block_start_ops: Vec<Option<&'a Operator<'a>>>,

    next_label_id: u32,

    /// Defines label variables.
    defs: DefinitionBuilder,
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

impl<'a> InstructionChainBuilder<'a> {
    fn single_instruction(instr: instruction::Instruction) -> InstructionChain {
        apply(instr, [unreachable()])
    }

    fn new(ops: &'a [Operator<'a>]) -> Self {
        Self {
            block_start_ops: Self::read_block_start_ops(ops),
            ..Default::default()
        }
    }

    fn read_block_start_ops(ops: &'a [Operator<'a>]) -> Vec<Option<&'a Operator<'a>>> {
        let mut stack = Vec::new();
        let mut block_start_ops = Vec::new();

        for op in ops.iter() {
            match op {
                Operator::Loop { .. } | Operator::Block { .. } | Operator::If { .. } => {
                    stack.push(op);
                }
                Operator::End => {
                    block_start_ops.push(stack.pop());
                }
                _ => {}
            }
        }

        block_start_ops
    }

    fn insert_label(&mut self, chain: InstructionChain) -> InstructionChain {
        let label_name = format!("_{:x}", self.next_label_id);
        self.next_label_id += 1;

        self.defs.def(label_name.clone(), chain);

        var(label_name)
    }

    fn push_end_label(&mut self, chain: InstructionChain) {
        self.labels.push(EndLabel {
            subchain: chain,
            block_start_op: self.block_start_ops.pop().unwrap(),
        });
    }

    fn pop_end_label(&mut self) -> EndLabel<'a> {
        self.labels.pop().unwrap()
    }

    fn push_else_label(&mut self, chain: InstructionChain) {
        self.else_labels.push(ElseLabel { subchain: chain });
    }

    fn pop_else_label(&mut self) -> ElseLabel {
        self.else_labels.pop().unwrap()
    }

    fn start_chain(&mut self, instr: instruction::Instruction) -> InstructionChain {
        let mut chain = unreachable();
        chain = self.insert_instruction(chain, instr);
        chain = self.insert_label(chain);
        self.push_end_label(chain.clone());
        chain
    }

    fn insert_instruction(
        &mut self,
        subchain: InstructionChain,
        instr: instruction::Instruction,
    ) -> InstructionChain {
        apply(instr, [subchain])
    }

    fn insert_label_if_needed(
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
}

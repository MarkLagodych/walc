use crate::{analyzer::*, codegen::*};

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
    let mut label_builder = LabelDefinitionBuilder::new(info.func.operators);

    let mut ops = info.func.operators.iter().rev();

    let function_end = ops.next().unwrap();

    let mut chain = unreachable();

    let instr = info.instrs.leave(info.func);
    chain = apply(instr, [chain]);
    chain = label_builder.insert_label_if_needed(chain, function_end);

    for op in ops {
        let instr = info
            .instrs
            .instruction(&mut instruction::InstructionBuildInfo {
                op,
                types: info.types,
                consts: info.consts,
                end_labels: &label_builder.end_labels,
                else_labels: &label_builder.else_labels,
            });

        chain = apply(instr, [chain]);
        chain = label_builder.insert_label_if_needed(chain, op);
    }

    let instr = info.instrs.enter(info.func, info.consts);
    chain = apply(instr, [chain]);

    label_builder.build(chain)
}

pub fn input_function(instrs: &mut instruction::InstructionDefinitionBuilder) -> InstructionChain {
    apply(instrs.input_and_return(), [unreachable()])
}

pub fn output_function(instrs: &mut instruction::InstructionDefinitionBuilder) -> InstructionChain {
    apply(instrs.output_and_return(), [unreachable()])
}

pub fn exit_function(instrs: &mut instruction::InstructionDefinitionBuilder) -> InstructionChain {
    apply(instrs.exit(), [unreachable()])
}

#[derive(Default)]
struct LabelDefinitionBuilder<'a> {
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
    fn new(ops: &'a [Operator<'a>]) -> Self {
        let mut me = Self::default();
        me.read_block_start_ops(ops);
        me
    }

    fn build(self, chain: InstructionChain) -> InstructionChain {
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

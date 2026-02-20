use super::*;

use crate::analyzer::*;

pub type FunctionBody = Expr;

pub struct FunctionBodyBuilder<'a> {
    info: &'a FunctionInfo<'a>,
    consts: &'a mut number::ConstantDefinitionBuilder,
    instrs: &'a mut instruction::InstructionDefinitionBuilder,

    /// Stack of labels.
    labels: Vec<LabelInfo<'a>>,

    /// Stack of "else" labels.
    else_labels: Vec<Expr>,

    /// Stack of operators that open blocks.
    /// When reading `End` operators, pop the corresponding block opening operators from this stack
    block_opening_ops: Vec<Option<&'a Operator<'a>>>,

    next_label_id: u32,

    defs: DefinitionBuilder,
}

pub struct LabelInfo<'a> {
    /// A subchain of instructions that the label points to.
    expr: Expr,

    // If None, this label is the function body's end label.
    // Otherwise, this is the block opening operator corresponding to this label.
    block_start: Option<&'a Operator<'a>>,
}

fn read_blocks<'a>(ops: &'a [Operator<'a>]) -> Vec<Option<&'a Operator<'a>>> {
    let mut stack = Vec::new();
    let mut blocks = Vec::new();

    for op in ops.iter() {
        match op {
            Operator::Loop { .. } | Operator::Block { .. } | Operator::If { .. } => {
                stack.push(op);
            }
            Operator::End => {
                blocks.push(stack.pop());
            }
            _ => {}
        }
    }

    blocks
}

impl<'a> FunctionBodyBuilder<'a> {
    pub fn new(
        info: &'a FunctionInfo<'a>,
        consts: &'a mut number::ConstantDefinitionBuilder,
        instrs: &'a mut instruction::InstructionDefinitionBuilder,
    ) -> FunctionBodyBuilder<'a> {
        Self {
            info,
            consts,
            instrs,
            labels: Vec::new(),
            else_labels: Vec::new(),
            block_opening_ops: read_blocks(info.operators),
            next_label_id: 0,
            defs: DefinitionBuilder::new(),
        }
    }

    /// `chain` is the chain of instructions following the block end
    fn block_end(&mut self, chain: Expr) -> Expr {
        let label = format!("_{:x}", self.next_label_id);
        self.next_label_id += 1;

        self.defs.def(label.clone(), chain);
        self.labels.push(LabelInfo {
            expr: var(label.clone()),
            block_start: self.block_opening_ops.pop().unwrap(),
        });
        var(label)
    }

    fn block_else(&mut self, chain: Expr) -> Expr {
        let label = format!("_{:x}", self.next_label_id);
        self.next_label_id += 1;

        self.defs.def(label.clone(), chain);
        self.else_labels.push(var(label.clone()));
        var(label)
    }

    fn block_start(&mut self) -> Expr {
        self.labels.pop().unwrap().expr
    }

    pub fn build(mut self) -> FunctionBody {
        let mut expr = unreachable();

        for op in self.info.operators.iter().rev() {
            match op {
                Operator::Loop { .. } | Operator::Block { .. } | Operator::If { .. } => {
                    expr = self.block_start();
                }
                Operator::Else { .. } => {
                    expr = self.block_else(expr);
                }
                Operator::End => {
                    expr = self.block_end(expr);
                }
                _ => {}
            }

            let instr = self.instrs.instruction(
                op,
                self.info,
                self.consts,
                &self.labels,
                &self.else_labels,
            );

            expr = apply(instr, [expr]);
        }

        let instr = self
            .instrs
            .enter(self.info.function_type, self.info.local_types, self.consts);

        expr = apply(instr, [expr]);

        self.defs.build(expr)
    }
}

pub fn function(
    info: &FunctionInfo,
    consts: &mut number::ConstantDefinitionBuilder,
    instrs: &mut instruction::InstructionDefinitionBuilder,
) -> FunctionBody {
    FunctionBodyBuilder::new(info, consts, instrs).build()
}

pub fn input_function(instrs: &mut instruction::InstructionDefinitionBuilder) -> FunctionBody {
    let mut expr = unreachable();
    expr = apply(instrs.input_and_return(), [expr]);
    expr
}

pub fn output_function(instrs: &mut instruction::InstructionDefinitionBuilder) -> FunctionBody {
    let mut expr = unreachable();
    expr = apply(instrs.output_and_return(), [expr]);
    expr
}

pub fn exit_function(instrs: &mut instruction::InstructionDefinitionBuilder) -> FunctionBody {
    instrs.exit()
}

use super::*;

use crate::analyzer::*;

pub type FunctionBody = Expr;

pub struct FunctionBodyBuilder<'a> {
    info: &'a FunctionInfo<'a>,
    consts: &'a mut number::ConstantDefinitionBuilder,
    instrs: &'a mut instruction::InstructionDefinitionBuilder,

    /// Stack of variable expressions.
    /// Each variable is a "label" and stores a subchain of instructions in the function body.
    labels: Vec<Expr>,

    label_id: std::ops::RangeFrom<usize>,

    defs: DefinitionBuilder,
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
            label_id: 0..,
            defs: DefinitionBuilder::new(),
        }
    }

    /// `chain` is the chain of instruction following the block end
    fn block_end(&mut self, chain: Expr) -> Expr {
        let label = var(format!("_{}", self.label_id.next().unwrap()));
        self.defs.def(label.clone(), chain);
        self.labels.push(label.clone());
        label
    }

    fn block_start(&mut self) -> Expr {
        self.labels.pop().unwrap()
    }

    pub fn build(mut self) -> FunctionBody {
        // TODO return instruction after the body ends
        let mut expr = self.block_end(unreachable());

        for instr in self.info.instructions.iter().rev() {
            match instr {
                Operator::Block { .. } | Operator::Loop { .. } | Operator::If { .. } => {
                    expr = self.block_start();
                }
                Operator::End => {
                    expr = self.block_end(expr);
                }
                _ => {}
            }

            let instr = self
                .instrs
                .instruction(instr, self.info, self.consts, &self.labels);

            expr = apply(instr, [expr]);
        }

        // TODO setup instruction that reads params and inits locals

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

pub fn input_function() -> FunctionBody {
    unreachable()
}

pub fn output_function() -> FunctionBody {
    unreachable()
}

pub fn exit_function() -> FunctionBody {
    io_command::exit()
}

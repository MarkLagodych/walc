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

    /// `chain` is the chain of instructions following the block end
    fn block_end(&mut self, chain: Expr) -> Expr {
        let label = format!("_{}", self.label_id.next().unwrap());
        self.defs.def(label.clone(), chain);
        self.labels.push(var(label.clone()));
        var(label)
    }

    fn block_start(&mut self) -> Expr {
        self.labels.pop().unwrap()
    }

    pub fn build(mut self) -> FunctionBody {
        let mut expr = unreachable();

        // TODO track information of the kind of a block that "end" refers to, so that
        // the codegen can generate the correct jump (forward/backward)
        // This probably requires a preliminary forward pass over the instructions
        // Label info: [{ is_backward_jump, label_expr }]
        // OR! Rather build the function body in natural order and come up with how to fix everything

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
    instrs.input_and_return()
}

pub fn output_function(instrs: &mut instruction::InstructionDefinitionBuilder) -> FunctionBody {
    instrs.output_and_return()
}

pub fn exit_function(instrs: &mut instruction::InstructionDefinitionBuilder) -> FunctionBody {
    instrs.exit()
}

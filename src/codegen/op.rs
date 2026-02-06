use super::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValueRepr {
    I32,
    I64,
}

pub struct FunctionBuilder {
    ops: Vec<Expr>,
}

impl FunctionBuilder {
    pub fn new(param_count: usize, result_count: usize, local_reprs: &[ValueRepr]) -> Self {
        let mut me = Self { ops: Vec::new() };

        // TODO

        me
    }

    pub fn build(mut self) -> Result<Expr> {
        let mut result = self.ops.pop().ok_or(anyhow!("Empty function"))?;

        for op in self.ops.into_iter().rev() {
            result = apply(op, [result]);
        }

        Ok(result)
    }
}

pub fn local() -> Expr {
    todo!()
}

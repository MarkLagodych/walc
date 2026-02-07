use super::*;

use crate::analyzer::*;

pub struct FunctionBuilder {}

impl FunctionBuilder {}

pub type FunctionBody = Expr;

pub fn function(
    param_count: u32,
    has_result: bool,
    local_types: &[ValType],
    operators: &[Operator],
) -> FunctionBody {
    unreachable()
}

pub fn input_function() -> FunctionBody {
    unreachable()
}

pub fn output_function() -> FunctionBody {
    unreachable()
}

pub fn exit_function() -> FunctionBody {
    unreachable()
}

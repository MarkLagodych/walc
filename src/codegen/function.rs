use super::*;

use crate::analyzer::*;

pub type FunctionBody = Expr;

// TODO get refs to consts, instrs, arith, and func type map
pub fn function(
    param_count: u32,
    has_result: bool,
    local_types: &[ValType],
    instructions: &[Operator],
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
    walc_io::end()
}

use super::*;

use crate::analyzer::*;

pub type FunctionBody = Expr;

pub fn function(
    info: &FunctionInfo,
    consts: &mut number::ConstantDefinitionBuilder,
) -> FunctionBody {
    let mut labels = Vec::<Operator>::new();

    unreachable()
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

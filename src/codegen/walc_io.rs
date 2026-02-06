use super::*;

pub fn end() -> Expr {
    var("End")
}

pub fn output(out_byte: Expr, next: Expr) -> Expr {
    apply(var("Out"), [out_byte, next])
}

pub fn input(root_input_handler: Expr) -> Expr {
    apply(var("In"), [root_input_handler])
}

pub(super) fn define_prelude(b: &mut DefinitionBuilder) {
    b.def("End", optional::none());

    b.def(
        "Out",
        abs(
            ["out_byte", "next"],
            optional::some(either::first(pair::new(var("out_byte"), var("next")))),
        ),
    );

    b.def(
        "In",
        abs(
            ["input_handler"],
            optional::some(either::second(var("input_handler"))),
        ),
    );
}

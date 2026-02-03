use crate::lambda::{self, Lambda};

use wasmparser as wasm;

pub fn compile(source: &[u8]) -> Lambda {
    let mut b = lambda::prelude();

    b.def("arr", lambda::tree_list::default());
    b.def(
        "arr",
        lambda::tree_list::insert(
            lambda::var("arr"),
            lambda::number::u32_const(3),
            lambda::number::u8_const(b'X'),
        ),
    );
    b.def(
        "byte",
        lambda::tree_list::index(lambda::var("arr"), lambda::number::u32_const(4)),
    );

    b.build(lambda::walc_io::output(
        lambda::var("byte"),
        lambda::walc_io::end(),
    ))
}

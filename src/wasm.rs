use crate::lambda::{self, Lambda};

use wasmparser as wasm;

pub fn compile(source: &[u8]) -> Lambda {
    // let mut root = lambda::walc_io::end();
    // root = lambda::walc_io::output(lambda::number::u8_const(b'o'), root);
    // root = lambda::walc_io::output(lambda::number::u8_const(b'l'), root);
    // root = lambda::walc_io::output(lambda::number::u8_const(b'l'), root);
    // root = lambda::walc_io::output(lambda::number::u8_const(b'e'), root);
    // root = lambda::walc_io::output(lambda::number::u8_const(b'H'), root);

    let mut root = lambda::def(
        [
            ("arr", lambda::tree_list::default()),
            (
                "arr",
                lambda::tree_list::insert(
                    lambda::var("arr"),
                    lambda::number::u32_const(3),
                    lambda::number::u8_const(b'X'),
                ),
            ),
            (
                "byte",
                lambda::tree_list::index(lambda::var("arr"), lambda::number::u32_const(3)),
            ),
            // ("arr", lambda::var("Arr2")),
            // (
            //     "arr",
            //     lambda::apply(
            //         lambda::var("ArrInsert2"),
            //         [
            //             lambda::var("arr"),
            //             lambda::list::from([lambda::bit(false), lambda::bit(true)].into_iter()),
            //             lambda::number::u8_const(b'A'),
            //         ]
            //         .into_iter(),
            //     ),
            // ),
            // (
            //     "byte",
            //     lambda::apply(
            //         lambda::var("arr"),
            //         [lambda::bit(true), lambda::bit(true)].into_iter(),
            //     ),
            // ),
        ]
        .into_iter(),
        lambda::walc_io::output(lambda::var("byte"), lambda::walc_io::end()),
    );

    root = lambda::define_prelude(root);

    root
}

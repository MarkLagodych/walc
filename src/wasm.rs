use crate::lambda::{self, Lambda};

use wasmbin::{indices::*, instructions::*, sections::*, *};

fn find_main(m: &Module) -> FuncId {
    for section in &m.sections {
        if let Section::Export(exports) = section {
            let exports = exports.try_contents().unwrap();

            for export in exports {
                if export.name == "main"
                    && let ExportDesc::Func(func_id) = export.desc
                {
                    return func_id;
                }
            }
        }
    }

    panic!("no main function found");
}

pub fn compile(m: &Module) -> Lambda {
    // let mut root = lambda::walc_io::end();
    // root = lambda::walc_io::output(root, lambda::number::u8_const(b'o'));
    // root = lambda::walc_io::output(root, lambda::number::u8_const(b'l'));
    // root = lambda::walc_io::output(root, lambda::number::u8_const(b'l'));
    // root = lambda::walc_io::output(root, lambda::number::u8_const(b'e'));
    // root = lambda::walc_io::output(root, lambda::number::u8_const(b'H'));
    let mut root = lambda::walc_io::end();

    root = lambda::walc_io::output(root, lambda::number::u8_const(b'\n'));

    root = lambda::walc_io::output(
        root,
        lambda::array_tree::index(lambda::var("arr"), lambda::number::u32_const(3)),
    );

    root = lambda::define(
        root,
        "arr",
        lambda::array_tree::insert(
            lambda::var("arr"),
            lambda::number::u32_const(3),
            lambda::number::u8_const(b'X'),
        ),
    );
    root = lambda::define(root, "arr", lambda::array_tree::default());

    root = lambda::define_prelude(root);

    root
}

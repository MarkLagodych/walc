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
    let mut root = lambda::walc_command::end();
    root = lambda::walc_command::output(root, lambda::number::u8_const(b'o'));
    root = lambda::walc_command::output(root, lambda::number::u8_const(b'l'));
    root = lambda::walc_command::output(root, lambda::number::u8_const(b'l'));
    root = lambda::walc_command::output(root, lambda::number::u8_const(b'e'));
    root = lambda::walc_command::output(root, lambda::number::u8_const(b'H'));

    root = lambda::define_prelude(root);

    root
}

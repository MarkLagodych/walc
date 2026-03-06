use super::*;

// Avoid shadowing of codegen::memory by this module
use crate::codegen::memory;

/// WALC memory size is 2^32 bytes and WASM memory page size is 2^16 bytes,
/// so there are 2^16 pages.
const MEMORY_SIZE_IN_PAGES: u32 = 1 << 16;

pub fn memory_size(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("MemSize") {
        let memory_size = rt.num.i32_const(MEMORY_SIZE_IN_PAGES);
        rt.def("MemSize", {
            let mut b = InstructionBuilder::new();
            b.push([memory_size]);
            b.build()
        });
    }

    var("MemSize")
}

pub fn memory_grow(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("MemGrow") {
        let memory_size = rt.num.i32_const(MEMORY_SIZE_IN_PAGES);
        rt.def("MemGrow", {
            let mut b = InstructionBuilder::new();
            b.pop(["ngrow"]);
            b.push([memory_size]);
            b.build()
        });
    }

    var("MemGrow")
}

pub fn memory_init_with_data(
    rt: &mut RuntimeGenerator,
    data_segment: list::List,
    memory_offset: number::I32,
) -> Instruction {
    if !rt.has("MemInit") {
        let body = abs(["dat", "offset", "mem"], {
            select(
                list::is_not_empty(var("dat")),
                var("mem"),
                apply(
                    rec(var("_MemInit")),
                    [
                        list::get_tail(var("dat")),
                        math::increment(rt, var("offset")),
                        memory::insert(var("mem"), var("offset"), list::get_head(var("dat"))),
                    ],
                ),
            )
        });

        rt.def_rec("_MemInit", body);

        let body = abs(["dat", "offset"], {
            let mut b = InstructionBuilder::new();

            b.set_memory(apply(
                rec(var("_MemInit")),
                [var("dat"), var("offset"), b.memory()],
            ));

            b.build()
        });

        rt.def("MemInit", body);
    }

    apply(var("MemInit"), [data_segment, memory_offset])
}

pub fn memory_fill(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("MemFill") {
        let body = abs(["mem", "addr", "byte", "max"], {
            select(
                math::equal(rt, var("addr"), var("max")),
                apply(
                    rec(var("_MemFill")),
                    [
                        memory::insert(var("mem"), var("addr"), var("byte")),
                        math::increment(rt, var("addr")),
                        var("byte"),
                        var("max"),
                    ],
                ),
                var("mem"),
            )
        });

        rt.def("_MemFill", body);

        let body = {
            let mut b = InstructionBuilder::new();

            b.pop(["addr", "val", "len"]);

            // The maximum address at which to stop
            let max = math::add(rt, var("addr"), var("len"));

            let byte = math::i32_to_byte(rt, var("val"));

            b.set_memory(apply(
                rec(var("_MemFill")),
                [b.memory(), var("addr"), byte, max],
            ));

            b.build()
        };

        rt.def("MemFill", body);
    }

    var("MemFill")
}

pub fn memory_copy(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("MemCopy") {
        let body = abs(["mem", "dst", "src", "maxsrc"], {
            select(
                math::equal(rt, var("src"), var("maxsrc")),
                apply(
                    rec(var("_MemCopy")),
                    [
                        memory::insert(
                            var("mem"),
                            var("dst"),
                            memory::index(var("mem"), var("src")),
                        ),
                        math::increment(rt, var("dst")),
                        math::increment(rt, var("src")),
                        var("maxsrc"),
                    ],
                ),
                var("mem"),
            )
        });

        rt.def("_MemCopy", body);

        let body = {
            let mut b = InstructionBuilder::new();

            b.pop(["dst", "src", "len"]);

            // Maximum source index
            let maxsrc = math::add(rt, var("src"), var("len"));

            b.set_memory(apply(
                rec(var("_MemCopy")),
                [b.memory(), var("dst"), var("src"), maxsrc],
            ));

            b.build()
        };

        rt.def("MemCopy", body);
    }

    var("MemCopy")
}

/// Generates all `load` instructions: `i(32|64).load[(8|16|32)_(u|s)]`, e.g.
/// `i32.load`, `i64.load32_s`.
///
/// Args:
/// * `target_bits`: 32 or 64
/// * `source_bits`: 8, 16, 32, or 64. Must be <= `target_bits`.
pub fn i_load(
    rt: &mut RuntimeGenerator,
    memory_offset: u32,
    target_bits: u8,
    source_bits: u8,
    signed: bool,
) -> Instruction {
    let sign = if signed { "S" } else { "U" };
    let source_bytes = source_bits / 8;
    let target_bytes = target_bits / 8;
    let pad_bytes = target_bytes - source_bytes;

    let name = format!("I{target_bits}Load{source_bits}{sign}");

    if !rt.has(&name) {
        let body = abs(["offset"], {
            let mut b = InstructionBuilder::new();

            b.pop(["base_addr"]);

            b.def("addr0", math::add(rt, var("base_addr"), var("offset")));

            for i in 1..source_bytes {
                b.def(
                    format!("addr{i}"),
                    math::increment(rt, var(format!("addr{}", i - 1))),
                );
            }

            for i in 0..source_bytes {
                b.load(format!("b{i}"), var(format!("addr{i}")));
            }

            // WASM stores numbers in little endian.
            // Pad the bytes with zeroes at the end if needed and reverse them to construct
            // a big-endian byte sequence.

            let be_bytes = (0..source_bytes)
                .map(|i| var(format!("b{i}")))
                .chain(std::iter::repeat_n(number::null_byte(), pad_bytes as usize))
                .rev();

            let mut result = match target_bits {
                32 => number::make_i32(be_bytes),
                64 => number::make_i64(be_bytes),
                _ => unreachable!(),
            };

            if signed {
                result = math::sign_extend(rt, result, target_bits, source_bits);
            }

            b.push([result]);

            b.build()
        });

        rt.def(&name, body);
    }

    let offset = rt.num.i32_const(memory_offset);
    apply(var(name), [offset])
}

/// Generates all `store` instructions: `i(32|64).store[(8|16|32)]`, e.g.
/// `i32.store`, `i64.store32`.
///
/// Args:
/// * `source_bits`: 32 or 64
/// * `target_bits`: 8, 16, 32, or 64. Must be <= `source_bits`.
pub fn i_store(
    rt: &mut RuntimeGenerator,
    memory_offset: u32,
    source_bits: u8,
    target_bits: u8,
) -> Instruction {
    let target_bytes = target_bits / 8;

    let name = format!("I{source_bits}Store{target_bits}");

    if !rt.has(&name) {
        let body = abs(["offset"], {
            let mut b = InstructionBuilder::new();

            b.pop(["base_addr", "val"]);

            b.def("addr0", math::add(rt, var("base_addr"), var("offset")));

            for i in 1..target_bytes {
                b.def(
                    format!("addr{i}"),
                    math::increment(rt, var(format!("addr{}", i - 1))),
                );
            }

            b.def(
                "bytes",
                math::split_lowest_bits_to_be_bytes(rt, var("val"), source_bits, target_bits),
            );

            // Reversed because the byte list is BE but numbers are stored in LE
            for i in (0..target_bytes).rev() {
                b.def("b", list::get_head(var("bytes")));

                // Do not get the tail of the last byte, it will be empty anyway
                if i != 0 {
                    b.def("bytes", list::get_tail(var("bytes")));
                }

                b.store(var(format!("addr{i}")), var("b"));
            }

            b.build()
        });

        rt.def(&name, body);
    }

    let offset = rt.num.i32_const(memory_offset);
    apply(var(name), [offset])
}

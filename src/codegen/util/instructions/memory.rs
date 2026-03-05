use super::*;

// Avoid shadowing of codegen::memory by this module
use crate::codegen::memory;

impl UtilGenerator {
    /// WALC memory size is 2^32 bytes and WASM memory page size is 2^16 bytes,
    /// so there are 2^16 pages.
    const MEMORY_SIZE_IN_PAGES: u32 = 1 << 16;

    pub fn memory_size(&mut self) -> Instruction {
        if !self.has("MemSize") {
            let memory_size = self.num.i32_const(Self::MEMORY_SIZE_IN_PAGES);
            self.def("MemSize", {
                let mut b = InstructionBuilder::new();
                b.push([memory_size]);
                b.build()
            });
        }

        var("MemSize")
    }

    pub fn memory_grow(&mut self) -> Instruction {
        if !self.has("MemGrow") {
            let memory_size = self.num.i32_const(Self::MEMORY_SIZE_IN_PAGES);
            self.def("MemGrow", {
                let mut b = InstructionBuilder::new();
                b.pop(["ngrow"]);
                b.push([memory_size]);
                b.build()
            });
        }

        var("MemGrow")
    }

    pub fn memory_init_with_data(
        &mut self,
        data_segment: list::List,
        memory_offset: u32,
    ) -> Instruction {
        if !self.has("MemInit") {
            let body = abs(["dat", "offset", "mem"], {
                select(
                    list::is_not_empty(var("dat")),
                    var("mem"),
                    apply(
                        rec(var("_MemInit")),
                        [
                            list::get_tail(var("dat")),
                            self.num_increment(var("offset")),
                            memory::insert(var("mem"), var("offset"), list::get_head(var("dat"))),
                        ],
                    ),
                )
            });

            self.def_rec("_MemInit", body);

            let body = abs(["dat", "offset"], {
                let mut b = InstructionBuilder::new();

                b.set_memory(apply(
                    rec(var("_MemInit")),
                    [var("dat"), var("offset"), b.memory()],
                ));

                b.build()
            });

            self.def("MemInit", body);
        }

        let offset = self.num.i32_const(memory_offset);
        apply(var("MemInit"), [data_segment, offset])
    }

    /// Generates all `load` instructions: `i(32|64).load[(8|16|32)_(u|s)]`, e.g.
    /// `i32.load`, `i64.load32_s`.
    ///
    /// Args:
    /// * `target_bits`: 32 or 64
    /// * `source_bits`: 8, 16, 32, or 64. Must be <= `target_bits`.
    pub fn i_load(
        &mut self,
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

        if !self.has(&name) {
            let body = abs(["offset"], {
                let mut b = InstructionBuilder::new();

                b.pop(["base_addr"]);

                b.def("addr0", self.num_add(var("base_addr"), var("offset")));

                for i in 1..source_bytes {
                    b.def(
                        format!("addr{i}"),
                        self.num_increment(var(format!("addr{}", i - 1))),
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
                    result = self.num_sign_extend(result, target_bits, source_bits);
                }

                b.push([result]);

                b.build()
            });

            self.def(&name, body);
        }

        let offset = self.num.i32_const(memory_offset);
        apply(var(name), [offset])
    }

    /// Generates all `store` instructions: `i(32|64).store[(8|16|32)]`, e.g.
    /// `i32.store`, `i64.store32`.
    ///
    /// Args:
    /// * `source_bits`: 32 or 64
    /// * `target_bits`: 8, 16, 32, or 64. Must be <= `source_bits`.
    pub fn i_store(&mut self, memory_offset: u32, source_bits: u8, target_bits: u8) -> Instruction {
        let target_bytes = target_bits / 8;

        let name = format!("I{source_bits}Store{target_bits}");

        if !self.has(&name) {
            let body = abs(["offset"], {
                let mut b = InstructionBuilder::new();

                b.pop(["base_addr", "val"]);

                b.def("addr0", self.num_add(var("base_addr"), var("offset")));

                for i in 1..target_bytes {
                    b.def(
                        format!("addr{i}"),
                        self.num_increment(var(format!("addr{}", i - 1))),
                    );
                }

                b.def(
                    "bytes",
                    self.num_split_lowest_bits_to_be_bytes(var("val"), source_bits, target_bits),
                );

                // Reversed because the byte list is BE but numbers are stored in LE
                for i in (0..target_bytes).rev() {
                    b.def("b", list::get_head(var("bytes")));
                    b.def("bytes", list::get_tail(var("bytes")));
                    b.store(var(format!("addr{i}")), var("b"));
                }

                b.build()
            });

            self.def(&name, body);
        }

        let offset = self.num.i32_const(memory_offset);
        apply(var(name), [offset])
    }
}

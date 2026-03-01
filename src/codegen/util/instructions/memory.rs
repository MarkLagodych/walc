use super::*;

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

    pub fn i32_load(&mut self, offset: u32) -> Instruction {
        if !self.has("I32Load") {
            let body = abs(["offset"], {
                let mut b = InstructionBuilder::new();
                b.pop(["base_addr"]);

                for i in 1..4 {
                    b.def(
                        format!("addr{i}"),
                        self.num_increment(var(format!("addr{}", i - 1))),
                    );
                }

                for i in 0..4 {
                    b.load(format!("b{i}"), var(format!("addr{i}")));
                }

                b.push([number::make_i32((0..4).map(|i| var(format!("b{i}"))))]);

                b.build()
            });

            self.def("I32Load", body);
        }

        let offset = self.num.i32_const(offset);
        apply(var("I32Load"), [offset])
    }

    pub fn i64_load(&mut self, offset: u32) -> Instruction {
        if !self.has("I64Load") {
            let body = abs(["offset"], {
                let mut b = InstructionBuilder::new();
                b.pop(["base_addr"]);

                b.def("addr0", self.num_add(var("base_addr"), var("offset")));

                for i in 1..8 {
                    b.def(
                        format!("addr{i}"),
                        self.num_increment(var(format!("addr{}", i - 1))),
                    );
                }

                for i in 0..8 {
                    b.load(format!("b{i}"), var(format!("addr{i}")));
                }

                b.push([number::make_i64((0..8).map(|i| var(format!("b{i}"))))]);

                b.build()
            });

            self.def("I64Load", body);
        }

        let offset = self.num.i32_const(offset);
        apply(var("I64Load"), [offset])
    }
}

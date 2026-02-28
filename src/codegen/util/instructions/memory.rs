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
}

/// Defines the ID of the global that stores the current memory size.
///
/// Even though the real size of the memory is always 4 GiB, WASM requires the interpreter
/// to respect the initial size and growth deltas provided by the program and track the current
/// memory size. Doing otherwise makes existing memory allocators confused.
pub const CURRENT_MEMORY_SIZE_GLOBAL_ID: u16 = u16::MAX;

/// Defines the ID of the global that stores the trap handler.
///
/// The trap handler is an optional function.
pub const TRAP_HANDLER_GLOBAL_ID: u16 = u16::MAX - 1;

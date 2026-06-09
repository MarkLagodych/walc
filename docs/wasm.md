# WALC WebAssembly functions

## Exports

WALC recognizes the exported functions specified below.
Other exports are ignored.

### `main`

Rust signature: `fn()`

WASM signature: `(func)`

This is the mandatory entrypoint of the program.

### `handle_trap`

Rust signature: `fn(u32)`

WASM signature: `(func (param i32))`

This is an optional trap handler.

If `handle_trap` is not exported, all traps result in execution termination.
Otherwise, when a trap occurs, `handle_trap` is called with one of the following
codes:
- 0: reached the `unreachable` instruction
- 1: division/remainder error, which means one of the following:
    * division by zero
    * signed division overflow: $\frac{-2^{31}}{-1}$ and $\frac{-2^{63}}{-1}$
- 2: attempt to perform floating-point arithmetic

    FP arithmetic is not supported by WALC.

The list of codes may be extended in the future.

#### Not handled

Other traps are currently **not** handled for efficiency:
- address out of range in `xxx.load` and `xxx.store`
- index out of range in `call_indirect`
- access of uninitialized table element
- etc.

The following conditions are not considered traps:
- calling `walc.exit`
- returning from the `main` function

#### Behavior

The memory and all the globals are preserved since the moment when the last
trap occurred.

The data stack is cleared and all the call frames (including all the locals)
are dropped before calling `handle_trap`.

Returning from `handle_trap` ends the program.

If `handle_trap` is called by the program itself, it behaves as a normal
function.

If a trap occurs within `handle_trap`, it gets called again.
Thus, one should be cautious when implementing `handle_trap` so as to avoid
infinite loops.

## Imports

WALC allows only the imports given below in WebAssembly modules.

Implementations of these functions are provided by the compiler and are
built into the resulting program.
The implementations use the corresponding WALC I/O commands as per
the [WALC lambda calculus spec](./text-format.md).

### `walc.input`

Rust signature: `fn() -> u32`

WASM signature: `(func (result i32))`

Reads a byte from STDIN.
In case of EOF, returns `0xFFFFFFFF`.

### `walc.output`

Rust signature: `fn(u8)`

WASM signature: `(func (param i32))`

Prints the given byte to STDOUT.

### `walc.exit`

Rust signature: `fn()`

WASM signature: `(func)`

Exits the program.

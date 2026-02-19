# WALC WebAssembly functions

WALC allows only the following imports in WebAssembly modules:
* `walc.input`

    Type: `() -> (i32)`

    Signature: `(function (result i32))`

    Reads a byte from STDIN.
    In case of EOF, returns `0xFFFFFFFF`.

* `walc.output`

    Type: `(i8) -> ()`

    Signature: `(function (param i32))`

    Prints the given byte to STDOUT.

* `walc.exit`

    Type: `() -> ()`

    Signature: `(function)`

    Exits the program.

Implementations of these functions are provided by the compiler and are
built into the resulting program.
The implementations use the corresponding WALC commands as per
the [format](./format.md).

# WebAssembly to Lambda Calculus compiler

WALC /wɑlts/ compiles stand-alone modules in [WebAssembly 1.0](https://w3.org/TR/wasm-core-1/)
([pdf](https://webassembly.github.io/spec/versions/core/WebAssembly-1.0.pdf))
into [untyped lambda expressions](https://en.wikipedia.org/wiki/Lambda_calculus).

The compiler supports [LIME1](https://github.com/WebAssembly/tool-conventions/blob/main/Lime.md)
WebAssembly extensions, but does not support floating-point arithmetic.
Floats are stored as integers, reinterpreting conversions between floats and
integers are replaced with nops and any other operations are replaced with
traps. This might be useful when you use a standard function like `printf`
that can use floats internally, but your program never invokes it with any
float values.

The input modules are only allowed to use custom [WALC functions](./docs/wasm.md)
for standard I/O, see [example programs](./examples/rust/)
written in Rust.

The output lambda expressions are in [WALC format](./docs/format.md),
which defines how the expressions should be interpreted in order to perform I/O.
The format does not change anything about lambda calculus or how it is evaluated
inside the interpreter, it just defines what the interpreter does with
the evaluated result.
You can run some [example expressions](./examples/walc/) written by hand
with an [example interpreter](./examples/interpreter/) available in Lua and
TypeScript.

Enjoy!

## Build & run

```sh
cargo build
cargo run -- INPUT.wasm -o OUTPUT.walc
```

or install it globally:

```sh
cargo install
walc INPUT.wasm -o OUTPUT.walc
```

## Examples

Example Rust programs are [here](./examples/rust/).

### Build

This projects ships with pre-built binaries, so you can typically skip this.

1. Install the WASM toolchain for Rust:
    ```sh
    rustup target add wasm32v1-none
    ```
    You can also experiment with the standard `wasm32-unknown-unknown` toolchain,
    but its feature set is unstable and in the future it might extend beyond
    what WALC supports.
2. Run the `make` command.
    This will install all the `.wasm` files into the `bin` directory.
    ```sh
    make
    ```

### Run

Example:

```sh
walc examples/bin/mandelbrot.wasm -o examples/target/mandelbrot.walc
examples/interpreter/lambda.ts examples/target/mandelbrot.walc
```

Output:

```
              ..............................:::::!?:!!:............
           ...............................:::::::!?@!:::::............
         ..............................:::::::?@@@@@@?!::::::...........
       .............................::::::::::?@@@@@@@!:::::::::..........
      ..........................:::::??@!::@@??@@@@@@@??!@:::::@::.........
    ......................::::::::::::@@@@@@@@@@@@@@@@@@@@@?@@@@!::..........
   ..................:::::::::::::::?!@@@@@@@@@@@@@@@@@@@@@@@@@!::::..........
  ...............::!:::::::::::::::?@@@@@@@@@@@@@@@@@@@@@@@@@@@@::::...........
  ............::::::@!!!:!@!:::::!?@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@::...........
 ..........::::::::::?@@@@@@@@@?!?@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@??::............
 ........::::::::::!@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@?:::............
 ..:...:::::::::!@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@?::::............
:?@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@?!:::::............
 ..:...:::::::::!@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@?::::............
 ........::::::::::!@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@?:::............
 ..........::::::::::?@@@@@@@@@?!?@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@??::............
  ............::::::@!!!:!@!:::::!?@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@::...........
  ...............::!:::::::::::::::?@@@@@@@@@@@@@@@@@@@@@@@@@@@@::::...........
   ..................:::::::::::::::?!@@@@@@@@@@@@@@@@@@@@@@@@@!::::..........
    ......................::::::::::::@@@@@@@@@@@@@@@@@@@@@?@@@@!::..........
      ..........................:::::??@!::@@??@@@@@@@??!@:::::@::.........
       .............................::::::::::?@@@@@@@!:::::::::..........
         ..............................:::::::?@@@@@@?!::::::...........
           ...............................:::::::!?@!:::::............
```

Note that this particular example typically takes around 15 minutes or so
to run.

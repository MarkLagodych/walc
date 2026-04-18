# WebAssembly to Lambda Calculus compiler

WALC /wɑlts/ compiles stand-alone modules in [WebAssembly](https://webassembly.org/)
into pure [untyped lambda expressions](https://en.wikipedia.org/wiki/Lambda_calculus).

The input modules are only allowed to use custom [WALC functions](./docs/wasm.md)
to input a byte, output a byte, and exit, see [example programs](./examples/rust/)
written in Rust.

The output lambda expressions are in human-readable [WALC format](./docs/text-format.md),
which just uses square brackets instead of the lambda symbol.
There is even a [one-line script](./tools/text2math) to convert it to the
standard mathematical notation.

All lambda calculus semantics and purity is preserved.
In order to perform I/O, the interpreter decodes the program as an I/O command,
executes the command, supplies encoded user input if needed, and repeats again.

See [example interpreters](./tools/) written in Lua and
TypeScript in under 300 LOC that are optimized for running lambda calculus
for a long time without speed and memory usage degradation.

You can run some [example lambda expressions](./examples/walc/) with:
```sh
interp/lambda.ts examples/walc/hello.walc
```

You might also utilize [overview notes](./docs/notes.md) as a starting point for
digging into the codebase.

Enjoy!

## Build & run

```sh
cargo run -- INPUT.wasm -o OUTPUT.walc
```

or install it globally:

```sh
cargo install --path .
walc INPUT.wasm -o OUTPUT.walc
```

## Examples

Example Rust programs are [here](./examples/rust/).

### Build

1. Install the WASM toolchain for Rust:
    ```sh
    rustup target add wasm32v1-none
    ```
    You can also experiment with the standard `wasm32-unknown-unknown`
    toolchain, even though its feature set is unstable and in the future it
    might extend beyond what WALC supports:
    ```sh
    rustup target add wasm32-unknown-unknown
    ```
2. Build for release. You can use the provided Makefile that will tell Cargo
    to also install all the `.wasm` files into the `examples/rust/bin`
    directory:
    ```sh
    make -C examples/rust
    ```

    Or, for the `wasm32-unknown-unknown` target:
    ```sh
    make -C examples/rust TARGET=wasm32-unknown-unknown
    ```

### Run

Using the TypeScript or Lua interpreters:

```sh
mkdir bin
walc examples/rust/bin/mandelbrot.wasm -o bin/mandelbrot.walc

tools/lambda.ts bin/mandelbrot.walc
```

*The TypeScript version runs in about 15 minutes on my machine.*

Using the C interpreter:
```sh
mkdir bin
walc examples/rust/bin/mandelbrot.wasm -o bin/mandelbrot.walc

gcc tools/lambda.c -o bin/lambda -O3
tools/text2bin.ts bin/mandelbrot.walc -o bin/mandelbrot.bin
bin/lambda bin/mandelbrot.bin
```

*The C version runs in about 8 minutes on my machine.*

*Both versions require around 1.4 GB of memory.* 🤷

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

## Feature support

WALC supports:
- [WebAssembly 1.0](https://w3.org/TR/wasm-core-1/)
    ([pdf](https://webassembly.github.io/spec/versions/core/WebAssembly-1.0.pdf)),
    the WWW standard released in 2019
- [Linear Memory 1.0](https://github.com/WebAssembly/tool-conventions/blob/main/Lime.md)
    extensions

WALC does not support:

- Dynamic type checking and bounds checking.

    Only division by zero and signed division overflow are checked.
    Other checks are ignored for efficiency, even though this is non-compliant
    behavior.

- Floating-point arithmetic.

    Given the scope of the project, there is simply no point in implementing
    this.

    To avoid as much compilation problems as possible,
    floats are stored as integers.
    Reinterpreting conversions between floats and integers are replaced with
    nops and any other operations are replaced with traps.
    This behavior might be useful when you use a standard function like `printf`
    that can use floats internally, but your program never invokes it with any
    float values.

# WebAssembly to Lambda Calculus compiler

WALC /wɑlts/ compiles stand-alone modules in [WebAssembly 1.0](https://w3.org/TR/wasm-core-1/)
([pdf](https://webassembly.github.io/spec/versions/core/WebAssembly-1.0.pdf))
into [untyped lambda expressions](https://en.wikipedia.org/wiki/Lambda_calculus).

The input modules are only allowed to use custom [WALC functions](./docs/wasm.md)
for standard I/O, see [example programs](./examples/rust/) written in Rust.

The output lambda expressions are in [WALC format](./docs/format.md),
which just uses square brackets instead of the lambda symbol.
It does not introduce any impurities to lambda calculus for I/O.
Instead, it only specifies how evaluated expressions must be further interpreted
as I/O commands instead of just plainly printed.

If you want, you can even convert WALC syntax to the standard mathematical
notation with:

```sh
cat INPUT.walc | sed -r 's/\[([_0-9a-zA-Z]+)/(λ\1./g ; s/]/)/g'
# Example input: [f ([x (f (x x))] [x (f (x x))])]
# Example output: (λf. ((λx. (f (x x))) (λx. (f (x x)))))
```

You can run some [example expressions](./examples/walc/) written by hand
with an [example interpreter](./examples/interpreter/) available in Lua and
TypeScript:
```sh
examples/interpreter/lambda.ts examples/walc/hello.walc
```

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
    You can also experiment with the standard `wasm32-unknown-unknown` toolchain,
    but its feature set is unstable and in the future it might extend beyond
    what WALC supports.
2. Build for release. You can use the provided Makefile that will tell Cargo
    to also install all the `.wasm` files into the `examples/rust/bin`
    directory:
    ```sh
    make -C examples/rust
    ```

### Run

Example:

```sh
walc examples/rust/bin/mandelbrot.wasm -o examples/rust/bin/mandelbrot.walc
examples/interpreter/lambda.ts examples/rust/bin/mandelbrot.walc
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

## Feature support

WALC supports:
- [WebAssembly 1.0](https://w3.org/TR/wasm-core-1/)
    ([pdf](https://webassembly.github.io/spec/versions/core/WebAssembly-1.0.pdf))
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

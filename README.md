# WebAssembly to Lambda Calculus compiler

WALC /wɑlts/ compiles stand-alone modules in [WebAssembly](https://webassembly.org/)
into pure closed untyped [lambda expressions](https://en.wikipedia.org/wiki/Lambda_calculus).

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

See [example interpreters](./tools/) written in Lua and TypeScript in under
300 LOC or in C in under 850 LOC that are optimized for running lambda calculus
for a long time with stable speed and reasonable memory consumption.

You can run some [example lambda expressions](./examples/walc/) with:
```sh
tools/lambda.ts examples/walc/hello.walc
```

You might also utilize [overview notes](./docs/notes.md) as a starting point for
digging into the codebase.

Enjoy!

## Build & run

Run from the project directory:

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
    rustup target add wasm32-unknown-unknown
    ```
    Note that the feature set of `wasm32-unknown-unknown` is unstable and
    in the future it might extend beyond what WALC supports.
    In that case, use `wasm32v1-none`.

2. Build:
    ```sh
    cd examples/rust

    # This will build in the "bin" directory
    cargo install -q --path . --root . --no-track --profile release --target wasm32-unknown-unknown
    ```

### Run

- You can use the [runwasm](./tools/runwasm) script:
    ```sh
    tools/runwasm examples/rust/bin/mandelbrot.wasm
    ```
    This is a shorthand for running the C interpreter, which requires
    a C compiler (`cc`) for the interpreter itself
    and also [Deno](https://deno.com/) for the [pre-parser script](./tools/text2bin.ts).
- Alternatively, you can run everything manually.
    The commands here are run from the root directory.

    + To use the C interpreter, run:
        ```sh
        mkdir -p bin
        walc examples/rust/bin/mandelbrot.wasm -o bin/mandelbrot.walc

        # Compile the interpreter
        gcc tools/lambda.c -o bin/lambda -O3
        # Pre-compile the lambda expression
        tools/text2bin.ts bin/mandelbrot.walc -o bin/mandelbrot.bin
        # Run!
        bin/lambda bin/mandelbrot.bin
        ```
    + To use the TypeScript/Lua interpreters, run:

        ```sh
        mkdir -p bin
        walc examples/rust/bin/mandelbrot.wasm -o bin/mandelbrot.walc

        tools/lambda.ts bin/mandelbrot.walc
        # or:
        tools/lambda.lua bin/mandelbrot.walc
        ```

Just for comparison, here are some approximate performace data from
running the [Mandelbrot example](./examples/rust/mandelbrot.rs) on my machine:

| Interpreter    | Compiler/Runtime   | Execution time | Peak memory usage |
|----------------|--------------------|----------------|-------------------|
| lambda.c 1.0   | GCC 13.3 (-O3)     | 4 min          | 75 MB             |
| lambda.ts 1.0  | Deno 2.7           | 15 min         | 400 MB            |
| lambda.lua 1.0 | LuaJIT 2.1         | 106 min (*)    | >900 MB           |

(*) Lua execution time is extrapolated from running half of the program for
53 min. 🤷

*While this might seem underwhelming, note that the interpreter was not the main
focus of this project and it took quite a bit of optimization to achieve
even this performance. I would love to hear about more efficient approaches! 🧑‍🔬
Who knows, maybe graph reduction techniques or conversion to combinatory
calculus might do a 10x speedup. Or sophisticated compiler optimizations?*

### Gallery

#### Mandelbrot fractal

[`mandelbrot.rs`](./examples/rust/mandelbrot.rs)

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

#### Tic-Tac-Toe

[`tictactoe.rs`](./examples/rust/tictactoe.rs)

```
Welcome to Tic-Tac-Toe! (^o^)/
    0   1   2
  +---+---+---+
0 |   |   |   |
  +---+---+---+
1 |   |   |   |
  +---+---+---+
2 |   |   |   |
  +---+---+---+
Your move (row column): 1 0
    0   1   2
  +---+---+---+
0 |   |   |   |
  +---+---+---+
1 | X |   |   |
  +---+---+---+
2 |   |   |   |
  +---+---+---+
My move... :-P
    0   1   2
  +---+---+---+
0 | O |   |   |
  +---+---+---+
1 | X |   |   |
  +---+---+---+
2 |   |   |   |
  +---+---+---+

(..........output shortened............)

    0   1   2
  +---+---+---+
0 | O | X | O |
  +---+---+---+
1 | X | X | O |
  +---+---+---+
2 | X | O | X |
  +---+---+---+
It's a draw! :-O
```

#### Sine and cosine

[`sincos.rs`](./examples/rust/sincos.rs)

```
Enter angle in radians: 1.047
sin: 0.866
cos: 0.500
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
    floats.

    To avoid as much compilation problems as possible,
    floats are stored as integers.
    Reinterpreting conversions between floats and integers are replaced with
    nops and all other operations are replaced with traps.
    This behavior might be useful when you use a standard function like `printf`
    that can use floats internally, but your program never invokes it with any
    float values.

## Spiritual inspiration

Or, in other words, you might also like to see:

- [C to Brainfuck compiler](https://github.com/arthaud/c2bf)
- [Esoteric language compiler infrastructure](https://github.com/shinh/elvm)
- [DOOM PDF port](https://github.com/ading2210/doompdf)
    and [Linux PDF port](https://github.com/ading2210/linuxpdf)
- [LazyK](https://tromp.github.io/cl/lazy-k.html)
    (see also the [wiki page](https://esolangs.org/wiki/Lazy_K))

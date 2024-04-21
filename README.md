# How to Use

```
Usage: compiler [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Specifies a path to an input .sp file. If None, it means stdin is used for input

Options:
  -o, --output <FILE>   Specifies a path to an output file. If None, it uses the input filename. If that's also None, it defaults to "output"
  -r, --run             Optionally runs and deletes the compiled executable. Only supported on Unix systems
  -h, --help            Print help
  -V, --version         Print version

Debug Options:
  -d, --display <PASS>  Specifies a pass to display [possible values: parse, validate, reveal, atomize, explicate, select]
  -t, --time            Print timing debug information
```

# Examples

These examples demonstrate how to build and run the compiler using Cargo.

Run with input from stdin:

```sh
echo "fn main() { print(42); }" | cargo run -- -r
```

Run with specified input path, without specifying an output path:

```sh
cargo run -- example.sp -r
```

Compile with specified input path and specified output path:

```sh
cargo run -- input.sp -o output
```

# Language Features

* Literals
* Var
* BinaryOp
* UnaryOp
* Let
* If
* Functions
    * Return
* Loop
    * Break
    * Continue
* While
* Sequences
* Structs

# AoC Wishlist for Christmas

* Pass raw strings for testing (allows testing AoC problems).
* Exprs instead of syms in asm
* And/or lazyness is ruined by atomize
* U8
* += *= etc.

# Fixes

* [ ] Updated README, with 3 new colors!
* [ ] Add documentation where necessary.
* [x] Improve error handling for parsing pass.
* [x] Improve error handling for type checking pass.
    * [ ] Make errors prettier.
* [x] Improve algorithm for colouring the interference graph.
* [x] Add read and write functionality to the bencher to update locally.
* [x] Lots, and lots, of refactoring!
* [x] Write test input in comments.
* [ ] Use arenas for improved performance TM

# Upcoming Language Features

* [x] Type inference.
* [x] Implement comments in code.
* [ ] Algebraic Data Types.
    * [x] Structs.
    * [ ] Enums.
* [ ] First-class functions.
* [ ] Constants.

# Upcoming Optimizations

* [ ] Dead code.
* [ ] Constant folding.
* [ ] Improve prologue and epilogue.
* [ ] Tail calls.
* [ ] And probably more...

# Lofty Goals

* [ ] Make the compiler suggest hints.
* [ ] Nested definitions.
* [ ] Match statements.
* [ ] LSP.
* [ ] GDB

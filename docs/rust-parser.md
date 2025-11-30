# Rust Parser Implementation

The C-Script project includes a parallel implementation of the parser written in Rust. This implementation uses the `nom` parser combinator library to provide a robust and efficient parsing experience.

## Overview

The Rust parser is located in the `rust/` directory. It is designed to parse C-Script source files and produce an Abstract Syntax Tree (AST) mirroring the structure used in the Python implementation.

## Project Structure

- **`rust/Cargo.toml`**: Defines the `c-script` package dependencies, including `nom` for parsing.
- **`rust/src/ast.rs`**: Contains the definitions of the AST nodes (e.g., `Program`, `Statement`, `Expression`). These are designed to be structurally similar to the Python nodes in `c_script/nodes.py`.
- **`rust/src/parser.rs`**: Implements the parsing logic using `nom`. It includes parsers for all language constructs such as variables, control flow (`if`, `while`, `for`), functions, and expressions.
- **`rust/src/bin/parser.rs`**: The command-line entry point. It reads a file, parses it, and prints the resulting AST (in debug format) to standard output.

## Usage

To run the Rust parser against a C-Script file, use `cargo run`:

```bash
cd rust
cargo run --bin parser -- <path_to_cscript_file>
```

Example:

```bash
cargo run --bin parser -- ../examples/test.cscript
```

## Relationship with Runtime

The runtime library (used for I/O and system interaction) is located separately in the `runtime/` directory. The Rust parser currently focuses solely on the parsing phase and does not yet generate code or link with the runtime, although the project structure supports future integration.

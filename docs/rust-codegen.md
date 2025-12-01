# Rust Code Generation

The C-Script project includes a parallel implementation of the code generator written in Rust. This component takes the AST produced by the Rust parser and generates LLVM IR (Intermediate Representation) text, which can then be compiled using standard LLVM tools.

## Overview

The Rust code generator is located in the `rust/` directory. It manually constructs LLVM IR strings, mirroring the logic of the Python `llvmlite`-based implementation but without external dependencies.

## Project Structure

- **`rust/src/codegen.rs`**: Implements the `CodeGen` struct. This module handles:
    - **Module & Target Setup**: Emits LLVM module headers and target triples.
    - **Runtime Declarations**: Declares external C-Script runtime functions (e.g., `cscript_print_int`, `cscript_fopen`).
    - **Function Generation**: Converts AST function definitions into LLVM functions, handling parameter allocation and stack storage.
    - **Statement & Expression Generation**: Recursively generates IR for all supported statements and expressions, managing temporary registers and labels for control flow.
- **`rust/src/bin/codegen.rs`**: The command-line entry point. It reads a source file, parses it using the `nom` parser, generates the LLVM IR, and prints it to standard output.

## Usage

To generate LLVM IR for a C-Script file:

```bash
cd rust
cargo run --bin codegen -- <path_to_cscript_file>
```

### Full Compilation Pipeline

To compile a C-Script file into an executable using the Rust toolchain:

1. **Generate LLVM IR**:
   ```bash
   cargo run --bin codegen -- ../examples/test.cscript > test.ll
   ```

2. **Compile to Object File** (using `llc`):
   ```bash
   llc -filetype=obj test.ll -o test.o
   ```
   *Note: You may need to use a specific version like `llc-18` or `llc-20` depending on your system.*

3. **Link with Runtime** (using `gcc`):
   ```bash
   gcc test.o ../runtime/target/release/libruntime.a -o test
   ```

4. **Run**:
   ```bash
   ./test
   ```

## Implementation Details

- **Manual IR Generation**: The generator uses string formatting to produce valid LLVM IR. This approach keeps the build lightweight.
- **Symbol Table**: The symbol table maps variable names to tuples of `(IR register name, Type)`. This enables:
  - **Type-Safe Code Generation**: Variables, parameters, and arrays maintain their type information throughout code generation
  - **Correct LLVM IR**: Load, store, and GEP (GetElementPtr) instructions use the actual declared types instead of assumptions
  - **Multi-Type Support**: Proper handling of `int`, `float`, `char`, and pointer types
  - **Array Type Tracking**: Arrays store their element type, enabling correct array access operations
- **Control Flow**: `if`, `while`, and `for` loops are implemented using basic blocks and conditional branches (`icmp`, `br`).
- **Runtime Integration**: The generated code calls into the static `runtime` library for I/O operations, ensuring compatibility with the existing C-Script ecosystem.

### Symbol Table Structure

The symbol table is implemented as:
```rust
HashMap<String, (String, Type)>
```

Where:
- **Key**: Variable name (e.g., `"x"`, `"arr"`)
- **Value**: Tuple of:
  - **IR Register/Pointer Name**: The LLVM temporary or pointer (e.g., `"%t0"`, `"%t5"`)
  - **Type**: The AST type information (`Type::Int`, `Type::Float`, `Type::Pointer(...)`, etc.)

This structure is populated during:
- **Function parameter allocation**: Parameters are stored with their declared types
- **Variable declarations**: Variables are stored with their declared types
- **Array declarations**: Arrays are stored with `Type::Pointer(element_type)` to track the element type

The type information is then used when:
- **Loading variables**: Generates the correct `load <type>, <type>* <ptr>` instruction
- **Storing to variables**: Generates the correct `store <type> <value>, <type>* <ptr>` instruction
- **Array access**: Generates the correct `getelementptr` instruction with proper element types

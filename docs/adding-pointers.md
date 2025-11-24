# Adding Pointer Support

This document details the implementation of pointer support in C-Script, including pointer types, address-of operator, and dereference operator.

## Overview

Support for pointers was added to allow direct memory manipulation and reference passing. This involved changes across the entire compiler pipeline: Lexer, Parser, AST Nodes, and Code Generator.

## Implementation Details

### 1. Lexer (`lexer.py`)

- Added a new token `AMPERSAND` for the `&` operator.
- The `*` operator was already present as `TIMES`.

### 2. AST Nodes (`nodes.py`)

- **`UnaryOp`**: A new node class was added to represent unary operations like `&x` and `*p`.
    ```python
    class UnaryOp:
        def __init__(self, op, operand):
            self.op = op
            self.operand = operand
    ```
- **`Assign`**: Refactored to support lvalue assignment. Instead of a string `name`, it now holds a `target` node (which can be an `Identifier` or a `UnaryOp` dereference).
    ```python
    class Assign:
        def __init__(self, target, value):
            self.target = target
            self.value = value
    ```

### 3. Parser (`parser.py`)

- **Types**: Updated the `type` rule to support recursive pointer types (e.g., `int*`, `int**`).
- **Expressions**: Added rules for unary operators `&` and `*` with `right` associativity and high precedence (`UNARY`).
- **Assignment**: Updated assignment rules to allow expressions on the left-hand side (lvalues), specifically identifiers and dereferences.

### 4. Code Generator (`codegen.py`)

- **Type Handling**: Implemented `_get_llvm_type` to recursively generate LLVM pointer types.
- **Unary Operations (`gen_unaryop`)**:
    - `&` (Address-of): Returns the `alloca` instruction (address) of the variable.
    - `*` (Dereference): Loads the value from the pointer address.
- **Assignment (`gen_assign`)**:
    - If the target is an `Identifier`, it behaves as before (store to variable's address).
    - If the target is a dereference (`*p`), it evaluates the operand `p` to get the address, then stores the value to that address.

## Example Usage

```c
def main() -> int {
    int x = 10;
    
    // Address-of
    int* p = &x;
    
    // Dereference (read)
    int y = *p;
    
    // Dereference (write)
    *p = 20;
    
    // Pointer to pointer
    int** pp = &p;
    
    return 0;
}
```

## Verification

The implementation was verified using `examples/test_pointers.cscript`, confirming that:
1. Pointers can be declared and initialized.
2. `&` correctly yields the address.
3. `*` correctly reads from and writes to the address.
4. Multi-level pointers work as expected.

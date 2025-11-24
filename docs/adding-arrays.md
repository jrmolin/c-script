# Adding Array Support

This document details the implementation of array support in C-Script, including fixed-size array declarations and element access.

## Overview

Array support was added to enable working with collections of values of the same type. This involved changes across the compiler pipeline: Lexer, Parser, AST Nodes, and Code Generator.

## Implementation Details

### 1. Lexer (`lexer.py`)

- Added `LBRACKET` (`[`) and `RBRACKET` (`]`) tokens for array syntax.

### 2. AST Nodes (`nodes.py`)

- **`ArrayDecl`**: A new node class for array declarations.
    ```python
    class ArrayDecl:
        def __init__(self, var_type, name, size):
            self.var_type = var_type
            self.name = name
            self.size = size
    ```
- **`ArrayAccess`**: A new node class for accessing array elements.
    ```python
    class ArrayAccess:
        def __init__(self, name, index):
            self.name = name
            self.index = index
    ```

### 3. Parser (`parser.py`)

- **Array Declaration**: Updated `var_declaration` to support `type ID LBRACKET NUMBER RBRACKET SEMI`.
- **Array Access**: Added expression rule for `ID LBRACKET expression RBRACKET`.
- **Uninitialized Variables**: Added support for `type ID SEMI` (variables without initialization), which defaults to 0. This was needed for loop counters.

### 4. Code Generator (`codegen.py`)

- **Array Declaration (`gen_arraydecl`)**:
    - Creates LLVM array type using `ir.ArrayType(element_type, size)`.
    - Allocates the array on the stack using `alloca`.
    - Stores the array pointer in the symbol table.

- **Array Access (`gen_arrayaccess`)**:
    - Uses `getelementptr` (GEP) instruction to calculate the address of the element.
    - Returns the loaded value for use in expressions.

- **Helper (`_get_array_ptr`)**:
    - Generates GEP with indices `[0, index]` to get the element pointer.
    - The first `0` dereferences the array pointer, the second is the element index.

- **Assignment to Arrays (`gen_assign`)**:
    - Extended to handle `ArrayAccess` as an lvalue.
    - Uses `_get_array_ptr` to get the element address, then stores the value.

## Example Usage

```c
def main() -> int {
    // Array declaration
    int x[5];
    int i;
    
    // Writing to array
    for (i = 0; i < 5; i = i + 1) {
        x[i] = i * 10;
    }
    
    // Reading from array
    for (i = 0; i < 5; i = i + 1) {
        print(x[i]);
    }
    
    // Random access
    x[2] = 99;
    print(x[2]);
    
    return 0;
}
```

## Verification

The implementation was verified using `examples/test_arrays.cscript`, confirming that:
1. Arrays can be declared with a fixed size.
2. Elements can be written using indexed assignment.
3. Elements can be read using indexed access.
4. Arrays work correctly in loops.

## Limitations

- **Fixed Size**: Array size must be a compile-time constant (number literal).
- **No Bounds Checking**: Out-of-bounds access is not checked at runtime.
- **No Dynamic Arrays**: Arrays cannot be resized after declaration.

# Malloc/Free with Runtime Bounds Checking

## Overview

Implemented memory allocation functions (`malloc` and `free`) with runtime bounds checking and error detection. This allows C-Script programs to dynamically allocate memory while being protected against common memory errors like double-free and use-after-free.

## Changes Made

### 1. Runtime Infrastructure ([lib.rs](file:///Users/mo/sandbox/llvm/c-script/runtime/src/lib.rs))

- **Allocation Table**: Added a global `ALLOCATIONS` map (using `Mutex` for thread safety) to track all active allocations.
- **`cscript_malloc`**: Allocates memory and records it in the table.
- **`cscript_free`**: Validates the pointer against the table.
  - Detects **Double Free**: If pointer is in table but marked invalid.
  - Detects **Invalid Pointer**: If pointer is not in table.
  - Marks allocation as invalid instead of removing it immediately (to distinguish double-free from not-allocated).
- **`cscript_check_bounds`**: Helper function to check if a pointer access is within bounds.

### 2. Code Generator ([codegen.rs](file:///Users/mo/sandbox/llvm/c-script/rust/src/codegen.rs))

- **Runtime Declarations**: Added declarations for `cscript_malloc`, `cscript_free`, and `cscript_check_bounds`.
- **Function Calls**: Mapped `malloc` and `free` calls to their runtime counterparts.
- **Pointer Operations**:
  - Implemented `UnaryOp::AddressOf` (`&x`) and `UnaryOp::Deref` (`*x`).
  - Updated assignment to support dereferenced targets (`*ptr = val`).
  - Fixed type handling for multi-level pointers (`**pp`).

### 3. Parser ([parser.rs](file:///Users/mo/sandbox/llvm/c-script/rust/src/parser.rs))

- **Recursive Unary Expressions**: Updated `parse_unary_expr` to be recursive, enabling `**ptr`.
- **Pointer Assignment**: Updated `parse_assign` to allow unary expressions on the LHS, enabling `*ptr = val`.

## Verification

### Basic Allocation ([test_malloc.cscript](file:///Users/mo/sandbox/llvm/c-script/examples/test_malloc.cscript))
```c
int* ptr = malloc(40);
*ptr = 42;
free(ptr);
```
✅ **Result**: Successfully allocates, writes, reads, and frees memory.

### Error Detection ([test_malloc_errors.cscript](file:///Users/mo/sandbox/llvm/c-script/examples/test_malloc_errors.cscript))
```c
free(ptr);
free(ptr); // Double free
```
✅ **Result**: Correctly detects double free:
```
free error: double free detected at address 0x...
```

### Pointer Operations ([test_pointers.cscript](file:///Users/mo/sandbox/llvm/c-script/examples/test_pointers.cscript))
```c
int** pp = &p;
print(**pp);
```
✅ **Result**: Correctly handles multi-level pointer dereferencing.

## Usage

```c
// Allocate memory
int* p = malloc(100);

// Use memory
*p = 123;

// Free memory
free(p);
```

The runtime will automatically print error messages to stderr if memory errors are detected.

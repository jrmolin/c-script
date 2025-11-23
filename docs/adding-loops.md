# Loop Support Implementation Walkthrough

I have successfully added support for `if`, `else`, `while`, and `for` loops to C-Script.

## Changes

### Lexer (`lexer.py`)
- Added keywords: `if`, `else`, `while`, `for`.
- Added comparison operators: `==`, `!=`, `<`, `>`, `<=`, `>=`.

### AST (`nodes.py`)
- Renamed `ast.py` to `nodes.py` to avoid conflict with Python's standard library `ast` module.
- Added `If`, `While`, `For` node classes.

### Parser (`parser.py`)
- Added grammar rules for control flow statements.
- Added precedence rules for operators to resolve conflicts.
- Updated imports to use `nodes.py`.

### CodeGen (`codegen.py`)
- Implemented `gen_if`, `gen_while`, `gen_for`.
- Updated `gen_binop` to support comparison operators.
- Fixed `gen_vardecl` to use 32-bit integers for `int` type, matching the spec.
- **Fix**: Updated target triple to `x86_64-pc-linux-gnu` to match the host system.

### Main (`main.py`)
- **Fix**: Added `-relocation-model=pic` flag to `llc` command to generate Position Independent Code (PIC) required for linking with GCC on Linux.

## Verification

I verified the implementation by running the compiler on `examples/loops.cscript`:

```c
int i = 0;
while (i < 5) {
    print(i);
    i = i + 1;
}

for (int j = 0; j < 3; j = j + 1) {
    print(j * 10);
}

if (i == 5) {
    print(100);
} else {
    print(200);
}
```

### Execution Output
```
0
1
2
3
4
0
10
20
100
```

The output confirms that:
1.  The `while` loop runs 5 times (0 to 4).
2.  The `for` loop runs 3 times (0, 10, 20).
3.  The `if` condition `i == 5` evaluates to true (since `i` was incremented to 5 in the while loop), printing 100.

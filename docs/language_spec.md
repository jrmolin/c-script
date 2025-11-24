# C-Script Language Specification

This document defines the syntax and semantics of the C-Script programming language.

## 1. Data Types

- `int`: 32-bit signed integer
- `float`: 32-bit floating-point number
- `char`: 8-bit signed integer

## 2. Variables

Variables are declared using the `int`, `float`, or `char` keywords, and can be assigned a value using the `=` operator.

Pointers are supported using the `*` suffix for types, `&` for address-of, and `*` for dereference.

Arrays are supported using the `[]` syntax for declaration and access.

```c
int x = 10;
int* p = &x;
*p = 20;

int arr[10];
arr[0] = 5;
```

```c
int x = 10;
float y = 3.14;
char z = 'a';

x = x + 1;
```

## 3. Control Flow

### 3.1. If-Else

```c
if (condition) {
  // ...
} else {
  // ...
}
```

### 3.2. For Loop

```c
for (i = 0; i < 10; i = i + 1) {
  // ...
}
```

### 3.3. While Loop

```c
while (condition) {
  // ...
}
```

## 4. User-Defined Functions

Functions are defined using the `def` keyword, followed by the function name, a list of parameters, the return type, and the function body.

```c
def add(int a, int b) -> int {
  return a + b;
}
```

### 4.1. Return Statement

The `return` statement is used to return a value from a function.

```c
return expression;
```

## 5. Modules and Imports

C-Script supports modular programming through the `import` statement. Standard library functions are organized into modules.

```c
import os
import file
```

## 6. Standard Library

### 6.1. Core Functions

The `print` function is available globally without imports.

```c
print(expression);
```

### 6.2. File Module (`import file`)

Provides functions for file input/output.

- `cscript_fopen(filename, mode)`: Opens a file. Mode can be "r" or "w". Returns a file handle (int).
- `cscript_fread(handle, size)`: Reads `size` bytes from the file. Returns the content as a string.
- `cscript_fwrite(handle, data)`: Writes `data` string to the file. Returns 1 on success.
- `cscript_fclose(handle)`: Closes the file.

```c
import file

int f = cscript_fopen("test.txt", "w");
cscript_fwrite(f, "Hello");
cscript_fclose(f);
```

### 6.3. OS Module (`import os`)

Provides functions for interacting with the operating system.

- `cscript_system(command)`: Executes a shell command. Returns the exit code.
- `cscript_getenv(name)`: Retrieves the value of an environment variable. Returns the value as a string.

```c
import os

cscript_system("ls -la");
print(cscript_getenv("HOME"));
```

# C-Script Language Specification

This document defines the syntax and semantics of the C-Script programming language.

## 1. Data Types

- `int`: 32-bit signed integer
- `float`: 32-bit floating-point number
- `char`: 8-bit signed integer

## 2. Variables

Variables are declared using the `int`, `float`, or `char` keywords, and can be assigned a value using the `=` operator.

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

## 5. Built-in Functions

### 5.1. Print

The `print` function prints the value of an expression to the console.

```c
print(expression);
```

### 5.2. File I/O

C-Script provides basic file I/O operations through a set of built-in functions.

- `fopen(filename, mode)`: Opens a file specified by `filename` in the given `mode` (e.g., "r", "w"). Returns a file handle (integer) on success, or -1 on error.
- `fread(handle, size)`: Reads `size` bytes from the file associated with `handle`. Returns a string containing the read data.
- `fwrite(handle, data)`: Writes the string `data` to the file associated with `handle`.
- `fclose(handle)`: Closes the file associated with `handle`.

```c
int handle = fopen("myfile.txt", "w");
fwrite(handle, "Hello, world!");
fclose(handle);
```

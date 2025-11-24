# Import Statement and Modules

C-Script supports modular programming through the `import` statement, allowing dynamic loading of runtime modules.

## Syntax

```c
import module_name
```

## Available Modules

### File Module (`import file`)

Provides functions for file input/output.

- `cscript_fopen(filename, mode)`: Opens a file. Mode can be "r" or "w". Returns a file handle (int).
- `cscript_fread(handle, size)`: Reads `size` bytes from the file. Returns the content as a string.
- `cscript_fwrite(handle, data)`: Writes `data` string to the file. Returns 1 on success.
- `cscript_fclose(handle)`: Closes the file.

### OS Module (`import os`)

Provides functions for interacting with the operating system.

- `cscript_system(command)`: Executes a shell command. Returns the exit code.
- `cscript_getenv(name)`: Retrieves the value of an environment variable. Returns the value as a string.

## Example Usage

```c
import os
import file

def main() -> int {
    print("Testing imports...");
    
    # OS Module usage
    print("Running system command...");
    cscript_system("echo 'Hello from system command'");
    
    print("Getting env var...");
    print(cscript_getenv("HOME"));
    
    # File Module usage
    print("Writing to file...");
    int f = cscript_fopen("test_output.txt", "w");
    cscript_fwrite(f, "Hello from file module");
    cscript_fclose(f);
    
    print("Reading from file...");
    int f2 = cscript_fopen("test_output.txt", "r");
    print(cscript_fread(f2, 100));
    cscript_fclose(f2);
    
    return 0;
}
```

## Implementation Details

- **Parser**: The `import` keyword is handled by the lexer and parser, creating an `Import` node in the AST.
- **CodeGen**: The code generator processes `Import` nodes to dynamically declare external functions in the LLVM module.
- **Runtime**: The runtime library (`lib.rs`) implements the actual functionality for these modules.

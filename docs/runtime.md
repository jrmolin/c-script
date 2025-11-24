# C-Script Runtime Library

The C-Script runtime library is a static library written in Rust that provides essential services to C-Script programs, such as input/output operations. It replaces the direct dependency on the C standard library for these functions, offering a safer and more manageable implementation.

## Architecture

The runtime is built as a static library (`libruntime.a`) located in the `rust/` directory. The C-Script compiler (`main.py`) builds this library using `cargo` and links it against the generated object files using `gcc`.

## Exported Functions

The runtime exports the following C-compatible functions (via `extern "C"`):

### Console I/O

- `void cscript_print_int(int val)`: Prints a 32-bit integer followed by a newline.
- `void cscript_print_float(float val)`: Prints a 32-bit float followed by a newline.
- `void cscript_print_string(char* val)`: Prints a null-terminated string followed by a newline.

### File I/O

- `int cscript_fopen(char* filename, char* mode)`: Opens a file. Returns a positive integer handle on success, or -1 on failure.
- `int cscript_fwrite(int handle, char* data)`: Writes a string to the file associated with `handle`. Returns 1 on success, 0 on failure.
- `char* cscript_fread(int handle, int size)`: Reads `size` bytes from the file associated with `handle`. Returns a pointer to a null-terminated string containing the data, or NULL on failure. **Note**: The returned string memory is currently leaked (for simplicity).
- `int cscript_fclose(int handle)`: Closes the file associated with `handle`. Returns 0 on success, -1 on failure.

## File Handle Management

To ensure safety and compatibility across different architectures (specifically regarding pointer sizes), the runtime uses a global **File Handle Table**.

- **Integer Handles**: Instead of passing raw `FILE*` pointers (which are 64-bit on 64-bit systems) to the 32-bit C-Script environment, the runtime maps open files to 32-bit integer handles.
- **Safety**: This prevents memory corruption issues where a 64-bit pointer might be truncated when stored in a 32-bit C-Script integer variable.
- **Implementation**: The table is implemented using a `Mutex<HashMap<i32, File>>` in Rust, ensuring thread safety (though C-Script is currently single-threaded).

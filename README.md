## C‑Script: A tiny programming language built with Python and LLVM

C‑Script is a minimal, C‑style toy language compiled with Python. It uses:
- PLY (lex/yacc) for tokenizing and parsing
- A small custom AST
- llvmlite to generate LLVM IR
- LLVM’s `llc` and a C toolchain (`gcc`/`clang`) to produce a native executable

This repo is intended for learning and experimentation: it’s compact, readable, and easy to extend.

### Features implemented
- **Scalars**: `int`, `float`, `char` (numbers are currently handled as integers in most operations)
- **Variables**: declarations and assignments
- **Expressions**: `+`, `-`, `*`, `/`, parentheses, identifiers, integer literals, string literals
- **Built‑ins**:
  - `print(expr)` prints integers and strings
  - Basic file I/O: `fopen(filename, mode)`, `fwrite(handle, data)`, `fclose(handle)`

See the in‑progress language notes in [`docs/language_spec.md`](docs/language_spec.md). Some constructs described there (e.g., control flow) are not implemented yet in the current parser.

## Requirements
- Python 3.13+
- LLVM toolchain with `llc`
- A C compiler (e.g., `gcc` or `clang`)

On macOS (Homebrew):

```bash
brew install llvm gcc
# You may need to expose LLVM tools first:
echo 'export PATH="/opt/homebrew/opt/llvm/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

## Install
Using pip:

```bash
python -m venv .venv
source .venv/bin/activate
pip install -e .
```

Using uv:

```bash
uv venv
source .venv/bin/activate
uv pip install -e .
```

Dependencies are defined in `pyproject.toml`:
- `ply`
- `llvmlite`

## Compile and run
The compiler is a Python CLI. It parses `.cscript`, emits LLVM IR, lowers to an object file via `llc`, and links a native binary.

```bash
python main.py examples/test.cscript -o hello
./hello
```

You should see:

```text
11
```

More examples:
```bash
python main.py examples/multiline.cscript -o multi && ./multi
python main.py examples/test_file_io.cscript -o fileio && ./fileio
```

## Language at a glance

### Arithmetic and grouping

```c
print(2 + 3 * (4 - 1));
```

### Multi‑line expressions

```c
print(1 +
       2 +
       3);
```

### Variables

```c
int x = 10;
x = x + 1;
print(x);
```

### Strings and file I/O (basic)

```c
int handle = fopen("test.txt", "w");
fwrite(handle, "hello world\n");
fclose(handle);
```

Notes:
- File handles are treated as opaque values for the language; under the hood they are interoperating with C `FILE*`.
- `print` supports integers and strings.

## How it works (architecture)
- `lexer.py`: token definitions via PLY
- `parser.py`: grammar rules that build an AST (`ast.py`)
- `ast.py`: simple node classes (`Program`, `VarDecl`, `Assign`, `Identifier`, `Number`, `String`, `BinOp`, `FuncCall`)
- `codegen.py`: translates AST to LLVM IR using `llvmlite`, declares `printf`, `fopen`, `fwrite`, `fclose`, and emits a `main` function
- `main.py`: CLI wrapper to parse, generate IR, run `llc`, and link with `gcc`

The pipeline is:
1) parse → 2) build AST → 3) generate LLVM IR → 4) `llc` to object → 5) link → 6) run

## Development
- Run the compiler directly from the repo:

```bash
python main.py examples/test.cscript -o a.out
./a.out
```

- Inspect the LLVM IR by pausing before cleanup (quick hack): comment out the cleanup lines at the end of `main.py` so that `*.ll` is kept.
- Extend the language by adding new AST nodes in `ast.py`, grammar rules in `parser.py`, and codegen in `codegen.py`.

## Troubleshooting
- “command not found: llc”: ensure LLVM is installed and `llc` is on your `PATH` (see macOS notes above).
- Linking errors: verify you have a working C toolchain (`gcc`/`clang`) and that the target triple in `codegen.py` is compatible with your platform.

## License
MIT (or the license of your choice)

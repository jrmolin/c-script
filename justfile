
alias b := build

build:
    uv run ./main.py ./examples/functions.cscript

run: build
    ./a.out

sync:
    uv sync

clean:
    rm -f a.out parser.out parsetab.py
    rm -rf __pycache__ rust/target

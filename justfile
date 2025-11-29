
alias b := build

build:
    uv run ./main.py ./examples/functions.cscript

run: build
    ./a.out

sync:
    uv sync

clean:
    rm -f a.out c_script/parser.out c_script/parsetab.py
    rm -rf __pycache__ c_script/__pycache__ rust/target
    rm -f test.txt

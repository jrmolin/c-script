set unstable

alias b := build

LLC := env("LLC", "") || "llc"
FILE := env("FILE", "") || "./examples/functions.cscript"

build :
    cargo run --manifest-path rust/Cargo.toml --bin codegen -- {{FILE}} -o a.out.ll


alias runtime := compile-runtime
compile-runtime:
    cargo build --release --manifest-path runtime/Cargo.toml

run: build compile-runtime
    {{LLC}} -relocation-model=pic -filetype=obj a.out.ll -o a.out.o
    gcc a.out.o runtime/target/release/libruntime.a -o a.out -lpthread -ldl
    ./a.out

sync:
    uv sync

clean:
    rm -f a.out c_script/parser.out c_script/parsetab.py
    rm -rf __pycache__ c_script/__pycache__ rust/target
    rm -f test.txt a.out.ll a.out.o

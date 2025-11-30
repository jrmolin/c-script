import os
import sys
import subprocess
import argparse

from c_script import lexer, parser, CodeGen

LLC = "llc"

path = os.environ["PATH"]

pathlist = path.split(os.pathsep)

for p in pathlist:
    for llc_ in ["llc", "llc-18", "llc-20"]:
        llc_test = os.path.join(p, llc_)

        if os.path.exists(llc_test):
            LLC = llc_test
            print(f"Using llc {LLC}")
            break

def main():
    arg_parser = argparse.ArgumentParser(description='C-Script compiler')
    arg_parser.add_argument('input', help='input file')
    arg_parser.add_argument('-o', '--output', help='output file',
                            default='a.out')
    arg_parser.add_argument('-d', '--debug', help="don't delete the output files",
                            action='store_true')
    arg_parser.add_argument('-r', '--run', help="run the output file",
                            action='store_true')
    args = arg_parser.parse_args()

    with open(args.input, 'r') as f:
        data = f.read()

    ast = parser.parse(data, lexer=lexer)
    codegen = CodeGen()
    codegen.generate(ast)

    ll_filename = args.output + '.ll'
    with open(ll_filename, 'w') as f:
        f.write(str(codegen.module))

    o_filename = args.output + '.o'
    subprocess.run([LLC, '-relocation-model=pic', '-filetype=obj', ll_filename, '-o', o_filename])

    # Build Rust runtime
    print("Building Rust runtime...")
    subprocess.run(['cargo', 'build', '--release', '--manifest-path', 'runtime/Cargo.toml'], check=True)

    # Link
    print("Linking...")
    runtime_lib = 'runtime/target/release/libruntime.a'
    subprocess.run(['gcc', o_filename, runtime_lib, '-o', args.output, '-lpthread', '-ldl'])

    if not args.debug:
        subprocess.run(['rm', ll_filename, o_filename])

    if args.run:
        subprocess.run([f"./{args.output}"])

if __name__ == "__main__":
    main()

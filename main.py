import sys
import subprocess
import argparse
from lexer import lexer
from parser import parser
from codegen import CodeGen


def main():
    arg_parser = argparse.ArgumentParser(description='C-Script compiler')
    arg_parser.add_argument('input', help='input file')
    arg_parser.add_argument('-o', '--output', help='output file',
                            default='a.out')
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
    subprocess.run(['llc', '-filetype=obj', ll_filename, '-o', o_filename])

    subprocess.run(['clang', o_filename, '-o', args.output])

    subprocess.run(['rm', ll_filename, o_filename])


if __name__ == "__main__":
    main()

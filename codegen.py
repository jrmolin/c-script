from llvmlite import ir

class CodeGen:
    def __init__(self):
        self.module = ir.Module(name="c-script")
        self.module.triple = "arm64-apple-darwin25.1.0"
        self.builder = None

        # Declare printf
        printf_ty = ir.FunctionType(ir.IntType(32), [ir.IntType(8).as_pointer()], var_arg=True)
        self.printf = ir.Function(self.module, printf_ty, name="printf")

        # Create a global format string for printf
        fmt = "%d\n\0"
        c_fmt = ir.Constant(ir.ArrayType(ir.IntType(8), len(fmt)), bytearray(fmt.encode("utf8")))
        self.global_fmt = ir.GlobalVariable(self.module, c_fmt.type, name="fstr")
        self.global_fmt.linkage = 'internal'
        self.global_fmt.global_constant = True
        self.global_fmt.initializer = c_fmt

    def generate(self, node):
        method = 'gen_' + node.__class__.__name__.lower()
        return getattr(self, method, self.generic_generate)(node)

    def generic_generate(self, node):
        raise Exception('No gen_{} method'.format(node.__class__.__name__.lower()))

    def gen_program(self, node):
        # Create a main function
        main_func_type = ir.FunctionType(ir.IntType(32), [])
        main_func = ir.Function(self.module, main_func_type, name="main")
        block = main_func.append_basic_block(name="entry")
        self.builder = ir.IRBuilder(block)

        # Generate code for each statement
        for stmt in node.stmts:
            self.generate(stmt)

        # Return 0
        self.builder.ret(ir.Constant(ir.IntType(32), 0))

    def gen_print(self, node):
        value = self.generate(node.expr)
        fmt_arg = self.builder.bitcast(self.global_fmt, ir.IntType(8).as_pointer())
        self.builder.call(self.printf, [fmt_arg, value])

    def gen_binop(self, node):
        lhs = self.generate(node.left)
        rhs = self.generate(node.right)

        if node.op == '+':
            return self.builder.add(lhs, rhs, name="addtmp")
        elif node.op == '-':
            return self.builder.sub(lhs, rhs, name="subtmp")
        elif node.op == '*':
            return self.builder.mul(lhs, rhs, name="multmp")
        elif node.op == '/':
            return self.builder.sdiv(lhs, rhs, name="divtmp")

    def gen_number(self, node):
        return ir.Constant(ir.IntType(32), node.value)

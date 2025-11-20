from llvmlite import ir

class CodeGen:
    def __init__(self):
        self.module = ir.Module(name="c-script")
        self.module.triple = "arm64-apple-darwin25.1.0"
        self.builder = None
        self.string_constants = {}
        self.symbol_table = {}

        # Declare C standard library functions
        self._declare_printf()
        self._declare_fopen()
        self._declare_fwrite()
        self._declare_fclose()

    def _declare_printf(self):
        printf_ty = ir.FunctionType(ir.IntType(32), [ir.IntType(8).as_pointer()], var_arg=True)
        self.printf = ir.Function(self.module, printf_ty, name="printf")

    def _declare_fopen(self):
        fopen_ty = ir.FunctionType(ir.IntType(8).as_pointer(), [ir.IntType(8).as_pointer(), ir.IntType(8).as_pointer()])
        self.fopen = ir.Function(self.module, fopen_ty, name="fopen")

    def _declare_fwrite(self):
        fwrite_ty = ir.FunctionType(ir.IntType(32), [ir.IntType(8).as_pointer(), ir.IntType(32), ir.IntType(32), ir.IntType(8).as_pointer()])
        self.fwrite = ir.Function(self.module, fwrite_ty, name="fwrite")

    def _declare_fclose(self):
        fclose_ty = ir.FunctionType(ir.IntType(32), [ir.IntType(8).as_pointer()])
        self.fclose = ir.Function(self.module, fclose_ty, name="fclose")

    def _get_string_constant(self, s):
        if s not in self.string_constants:
            fmt = s + "\0"
            c_fmt = ir.Constant(ir.ArrayType(ir.IntType(8), len(fmt)), bytearray(fmt.encode("utf8")))
            global_fmt = ir.GlobalVariable(self.module, c_fmt.type, name=f"fstr_{len(self.string_constants)}")
            global_fmt.linkage = 'internal'
            global_fmt.global_constant = True
            global_fmt.initializer = c_fmt
            self.string_constants[s] = global_fmt
        return self.string_constants[s]

    def generate(self, node):
        method = 'gen_' + node.__class__.__name__.lower()
        return getattr(self, method, self.generic_generate)(node)

    def generic_generate(self, node):
        if node is None:
            return
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

    def gen_vardecl(self, node):
        var_type = None
        if node.var_type == 'int':
            var_type = ir.IntType(64)
        elif node.var_type == 'float':
            var_type = ir.FloatType()
        elif node.var_type == 'char':
            var_type = ir.IntType(8)

        ptr = self.builder.alloca(var_type, name=node.name)
        self.symbol_table[node.name] = ptr
        value = self.generate(node.value)
        if isinstance(value.type, ir.PointerType) and isinstance(var_type, ir.IntType):
            value = self.builder.ptrtoint(value, var_type)
        self.builder.store(value, ptr)

    def gen_assign(self, node):
        ptr = self.symbol_table[node.name]
        value = self.generate(node.value)
        if isinstance(value.type, ir.PointerType) and isinstance(ptr.type.pointee, ir.IntType):
            value = self.builder.ptrtoint(value, ptr.type.pointee)
        self.builder.store(value, ptr)

    def gen_identifier(self, node):
        ptr = self.symbol_table[node.name]
        return self.builder.load(ptr, name=node.name)

    def gen_funccall(self, node):
        if node.name == 'print':
            value = self.generate(node.args[0])
            if isinstance(value.type, ir.IntType):
                fmt = "%d\n"
            else:
                fmt = "%s\n"
            fmt_arg = self.builder.bitcast(self._get_string_constant(fmt), ir.IntType(8).as_pointer())
            self.builder.call(self.printf, [fmt_arg, value])
        elif node.name == 'fopen':
            filename = self.generate(node.args[0])
            mode = self.generate(node.args[1])
            return self.builder.call(self.fopen, [filename, mode])
        elif node.name == 'fwrite':
            handle = self.generate(node.args[0])
            if isinstance(handle.type, ir.IntType):
                handle = self.builder.inttoptr(handle, ir.IntType(8).as_pointer())
            data = self.generate(node.args[1])
            size = ir.Constant(ir.IntType(32), len(node.args[1].value))
            self.builder.call(self.fwrite, [data, size, ir.Constant(ir.IntType(32), 1), handle])
        elif node.name == 'fclose':
            handle = self.generate(node.args[0])
            if isinstance(handle.type, ir.IntType):
                handle = self.builder.inttoptr(handle, ir.IntType(8).as_pointer())
            self.builder.call(self.fclose, [handle])

    def gen_string(self, node):
        return self.builder.bitcast(self._get_string_constant(node.value), ir.IntType(8).as_pointer())

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

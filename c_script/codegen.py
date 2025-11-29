from llvmlite import ir
import platform

linux_triple = "x86_64-pc-linux-gnu"
macos_arm_triple = "aarch64-apple-darwin"
macos_x86_triple = "x86_64-apple-darwin"

class CodeGen:
    def __init__(self):
        self.module = ir.Module(name="c-script")
        if platform.system().lower() == "linux":
            self.module.triple = linux_triple
        elif platform.system().lower() == "darwin":
            if platform.machine().lower() == "arm64":
                self.module.triple = macos_arm_triple
            else:
                self.module.triple = macos_x86_triple
        self.builder = None
        self.string_constants = {}
        self.symbol_table = {}

        # Declare C standard library functions (replaced by Rust runtime)
        self._declare_runtime_funcs()

    def _get_llvm_type(self, type_str):
        if type_str.endswith('*'):
            return self._get_llvm_type(type_str[:-1]).as_pointer()
        
        if type_str == 'int':
            return ir.IntType(32)
        elif type_str == 'float':
            return ir.FloatType()
        elif type_str == 'char':
            return ir.IntType(8)
        elif type_str == 'void':
            return ir.VoidType()
        else:
            raise Exception(f"Unknown type: {type_str}")

    def _declare_runtime_funcs(self):
        # void cscript_print_int(int)
        print_int_ty = ir.FunctionType(ir.VoidType(), [ir.IntType(32)])
        self.cscript_print_int = ir.Function(self.module, print_int_ty, name="cscript_print_int")

        # void cscript_print_float(float)
        print_float_ty = ir.FunctionType(ir.VoidType(), [ir.FloatType()])
        self.cscript_print_float = ir.Function(self.module, print_float_ty, name="cscript_print_float")

        # void cscript_print_string(char*)
        print_str_ty = ir.FunctionType(ir.VoidType(), [ir.IntType(8).as_pointer()])
        self.cscript_print_string = ir.Function(self.module, print_str_ty, name="cscript_print_string")

    def gen_import(self, node):
        module = node.module
        if module == "file":
            self._declare_file_funcs()
        elif module == "os":
            self._declare_os_funcs()
        else:
            raise Exception(f"Unknown module: {module}")

    def _declare_file_funcs(self):
        if hasattr(self, 'cscript_fopen'): return

        # void* cscript_fopen(char*, char*) -> int
        fopen_ty = ir.FunctionType(ir.IntType(32), [ir.IntType(8).as_pointer(), ir.IntType(8).as_pointer()])
        self.cscript_fopen = ir.Function(self.module, fopen_ty, name="cscript_fopen")

        # int cscript_fwrite(int, char*)
        fwrite_ty = ir.FunctionType(ir.IntType(32), [ir.IntType(32), ir.IntType(8).as_pointer()])
        self.cscript_fwrite = ir.Function(self.module, fwrite_ty, name="cscript_fwrite")

        # int cscript_fclose(int)
        fclose_ty = ir.FunctionType(ir.IntType(32), [ir.IntType(32)])
        self.cscript_fclose = ir.Function(self.module, fclose_ty, name="cscript_fclose")
        
        # char* cscript_fread(int, int)
        fread_ty = ir.FunctionType(ir.IntType(8).as_pointer(), [ir.IntType(32), ir.IntType(32)])
        self.cscript_fread = ir.Function(self.module, fread_ty, name="cscript_fread")

    def _declare_os_funcs(self):
        if hasattr(self, 'cscript_system'): return

        # int cscript_system(char*)
        system_ty = ir.FunctionType(ir.IntType(32), [ir.IntType(8).as_pointer()])
        self.cscript_system = ir.Function(self.module, system_ty, name="cscript_system")

        # char* cscript_getenv(char*)
        getenv_ty = ir.FunctionType(ir.IntType(8).as_pointer(), [ir.IntType(8).as_pointer()])
        self.cscript_getenv = ir.Function(self.module, getenv_ty, name="cscript_getenv")

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
        # Separate functions, imports, and other statements
        functions = []
        imports = []
        statements = []
        
        for stmt in node.stmts:
            if stmt.__class__.__name__ == 'FunctionDef':
                functions.append(stmt)
            elif stmt.__class__.__name__ == 'Import':
                imports.append(stmt)
            else:
                statements.append(stmt)

        # Process imports first to declare runtime functions
        for imp in imports:
            self.generate(imp)

        # Generate user functions
        for func in functions:
            self.generate(func)

        # Create a main function for top-level statements
        # If there are no top-level statements (other than imports), we might not need this
        # but existing logic seems to expect it.
        # However, if the user defined 'main', we shouldn't create another 'main'.
        # But the current implementation creates 'main' for top-level statements.
        # If the user defines 'main', it will conflict if we name this one 'main' too?
        # The existing code names it 'main'.
        # If the user defines 'main', the existing code generates it in 'functions'.
        # LLVM might complain about redefinition if we have two 'main's.
        # But let's stick to fixing the import order first.
        
        if statements:
            main_func_type = ir.FunctionType(ir.IntType(32), [])
            # If user defined main, we might have a problem. 
            # But let's assume top-level statements are put into a different entry point or 
            # the user is not supposed to mix 'def main' and top-level code.
            # For now, let's keep existing behavior but only if there are statements.
            
            # Check if main is already defined by user
            if "main" not in self.module.globals:
                 main_func = ir.Function(self.module, main_func_type, name="main")
                 block = main_func.append_basic_block(name="entry")
                 self.builder = ir.IRBuilder(block)
    
                 # Generate code for each statement
                 for stmt in statements:
                     self.generate(stmt)
    
                 # Return 0
                 self.builder.ret(ir.Constant(ir.IntType(32), 0))


    def gen_functiondef(self, node):
        # Return type
        ret_type = self._get_llvm_type(node.return_type)
            
        # Param types
        param_types = []
        for p_type, p_name in node.params:
            param_types.append(self._get_llvm_type(p_type))
                
        func_type = ir.FunctionType(ret_type, param_types)
        func = ir.Function(self.module, func_type, name=node.name)
        
        # Add arguments to symbol table
        entry_block = func.append_basic_block(name="entry")
        previous_builder = self.builder
        self.builder = ir.IRBuilder(entry_block)
        
        # Store params in alloca so they are mutable (if we want them to be)
        # and to match how we handle variables
        for i, (p_type, p_name) in enumerate(node.params):
            arg = func.args[i]
            arg.name = p_name
            alloca = self.builder.alloca(param_types[i], name=p_name)
            self.builder.store(arg, alloca)
            self.symbol_table[p_name] = alloca
            
        # Generate body
        for stmt in node.body:
            self.generate(stmt)
            
        # Restore builder
        self.builder = previous_builder

    def gen_return(self, node):
        value = self.generate(node.value)
        self.builder.ret(value)

    def gen_vardecl(self, node):
        var_type = self._get_llvm_type(node.var_type)

        ptr = self.builder.alloca(var_type, name=node.name)
        self.symbol_table[node.name] = ptr
        value = self.generate(node.value)
        if isinstance(value.type, ir.PointerType) and isinstance(var_type, ir.IntType):
            value = self.builder.ptrtoint(value, var_type)
        self.builder.store(value, ptr)

    def gen_arraydecl(self, node):
        element_type = self._get_llvm_type(node.var_type)
        array_type = ir.ArrayType(element_type, node.size)
        ptr = self.builder.alloca(array_type, name=node.name)
        self.symbol_table[node.name] = ptr

    def gen_arrayaccess(self, node):
        # This returns the VALUE (load) by default
        ptr = self._get_array_ptr(node)
        return self.builder.load(ptr)

    def _get_array_ptr(self, node):
        # Helper to get the pointer to the element
        array_ptr = self.symbol_table[node.name]
        index = self.generate(node.index)
        
        # GEP: [0, index]
        # We need two indices: 0 to dereference the array pointer, and index for the element
        zero = ir.Constant(ir.IntType(32), 0)
        return self.builder.gep(array_ptr, [zero, index], inbounds=True)

    def gen_assign(self, node):
        target = node.target
        ptr = None
        
        if isinstance(target, type(node) if False else object) and target.__class__.__name__ == 'Identifier':
            ptr = self.symbol_table[target.name]
        elif isinstance(target, type(node) if False else object) and target.__class__.__name__ == 'UnaryOp' and target.op == '*':
            # Dereference assignment: *p = val
            # Evaluate the operand to get the pointer address
            ptr = self.generate(target.operand)
        elif isinstance(target, type(node) if False else object) and target.__class__.__name__ == 'ArrayAccess':
            # Array assignment: x[i] = val
            ptr = self._get_array_ptr(target)
        else:
            raise Exception("Invalid lvalue for assignment")

        value = self.generate(node.value)
        # Type casting if needed (e.g. ptr to int)
        # This is a bit simplistic, but keeps existing behavior
        if isinstance(value.type, ir.PointerType) and isinstance(ptr.type.pointee, ir.IntType):
             # Only cast if we are assigning a pointer to an int variable (which is weird but existing code did it)
             # But wait, existing code was:
             # if isinstance(value.type, ir.PointerType) and isinstance(var_type, ir.IntType):
             #    value = self.builder.ptrtoint(value, var_type)
             # This was probably for string literals (char*) assigned to int?
             # Or maybe just safety.
             # Let's keep it but check types carefully.
             value = self.builder.ptrtoint(value, ptr.type.pointee)
             
        self.builder.store(value, ptr)

    def gen_unaryop(self, node):
        if node.op == '&':
            # Address-of
            operand = node.operand
            if isinstance(operand, type(node) if False else object) and operand.__class__.__name__ == 'Identifier':
                return self.symbol_table[operand.name]
            elif isinstance(operand, type(node) if False else object) and operand.__class__.__name__ == 'UnaryOp' and operand.op == '*':
                # &(*p) -> p
                return self.generate(operand.operand)
            else:
                raise Exception("Cannot take address of rvalue")
        elif node.op == '*':
            # Dereference
            ptr = self.generate(node.operand)
            return self.builder.load(ptr)

    def gen_identifier(self, node):
        ptr = self.symbol_table[node.name]
        return self.builder.load(ptr, name=node.name)

    def gen_funccall(self, node):
        if node.name == 'print':
            value = self.generate(node.args[0])
            if isinstance(value.type, ir.IntType):
                self.builder.call(self.cscript_print_int, [value])
            elif isinstance(value.type, ir.FloatType):
                self.builder.call(self.cscript_print_float, [value])
            else:
                # Assume string or char*
                self.builder.call(self.cscript_print_string, [value])
        elif node.name == 'fopen':
            filename = self.generate(node.args[0])
            mode = self.generate(node.args[1])
            return self.builder.call(self.cscript_fopen, [filename, mode])
        elif node.name == 'fwrite':
            handle = self.generate(node.args[0])
            data = self.generate(node.args[1])
            # cscript_fwrite takes (handle, data)
            self.builder.call(self.cscript_fwrite, [handle, data])
        elif node.name == 'fread':
            handle = self.generate(node.args[0])
            size = self.generate(node.args[1])
            return self.builder.call(self.cscript_fread, [handle, size])
        elif node.name == 'fclose':
            handle = self.generate(node.args[0])
            self.builder.call(self.cscript_fclose, [handle])
        else:
            # User defined function
            if node.name in self.module.globals:
                func = self.module.globals[node.name]
                args = [self.generate(arg) for arg in node.args]
                return self.builder.call(func, args)
            else:
                raise Exception(f"Function {node.name} not defined")

    def gen_string(self, node):
        return self.builder.bitcast(self._get_string_constant(node.value), ir.IntType(8).as_pointer())

    def gen_if(self, node):
        cond_val = self.generate(node.condition)
        # Convert i32 to i1 for branch
        cond_bool = self.builder.icmp_signed('!=', cond_val, ir.Constant(ir.IntType(32), 0), name="ifcond")
        
        then_bb = self.builder.append_basic_block(name="then")
        else_bb = self.builder.append_basic_block(name="else")
        merge_bb = self.builder.append_basic_block(name="ifcont")
        
        self.builder.cbranch(cond_bool, then_bb, else_bb)
        
        # Then block
        self.builder.position_at_start(then_bb)
        for stmt in node.then_body:
            self.generate(stmt)
        if not self.builder.block.is_terminated:
            self.builder.branch(merge_bb)
            
        # Else block
        self.builder.position_at_start(else_bb)
        if node.else_body:
            for stmt in node.else_body:
                self.generate(stmt)
        if not self.builder.block.is_terminated:
            self.builder.branch(merge_bb)
            
        # Merge block
        self.builder.position_at_start(merge_bb)

    def gen_while(self, node):
        cond_bb = self.builder.append_basic_block(name="whilecond")
        body_bb = self.builder.append_basic_block(name="whilebody")
        end_bb = self.builder.append_basic_block(name="whileend")
        
        self.builder.branch(cond_bb)
        
        # Condition block
        self.builder.position_at_start(cond_bb)
        cond_val = self.generate(node.condition)
        cond_bool = self.builder.icmp_signed('!=', cond_val, ir.Constant(ir.IntType(32), 0), name="whilecond")
        self.builder.cbranch(cond_bool, body_bb, end_bb)
        
        # Body block
        self.builder.position_at_start(body_bb)
        for stmt in node.body:
            self.generate(stmt)
        if not self.builder.block.is_terminated:
            self.builder.branch(cond_bb)
            
        # End block
        self.builder.position_at_start(end_bb)

    def gen_for(self, node):
        # Init
        self.generate(node.init)
        
        cond_bb = self.builder.append_basic_block(name="forcond")
        body_bb = self.builder.append_basic_block(name="forbody")
        end_bb = self.builder.append_basic_block(name="forend")
        
        self.builder.branch(cond_bb)
        
        # Condition block
        self.builder.position_at_start(cond_bb)
        cond_val = self.generate(node.condition)
        cond_bool = self.builder.icmp_signed('!=', cond_val, ir.Constant(ir.IntType(32), 0), name="forcond")
        self.builder.cbranch(cond_bool, body_bb, end_bb)
        
        # Body block
        self.builder.position_at_start(body_bb)
        for stmt in node.body:
            self.generate(stmt)
        
        # Update
        self.generate(node.update)
        
        if not self.builder.block.is_terminated:
            self.builder.branch(cond_bb)
            
        # End block
        self.builder.position_at_start(end_bb)

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
        elif node.op in ('<', '<=', '>', '>=', '==', '!='):
            cmp = self.builder.icmp_signed(node.op, lhs, rhs, name="cmptmp")
            return self.builder.zext(cmp, ir.IntType(32), name="booltmp")

    def gen_number(self, node):
        return ir.Constant(ir.IntType(32), node.value)




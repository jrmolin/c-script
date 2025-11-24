class Number:
    def __init__(self, value):
        self.value = value

class String:
    def __init__(self, value):
        self.value = value

class Identifier:
    def __init__(self, name):
        self.name = name

class BinOp:
    def __init__(self, left, op, right):
        self.left = left
        self.op = op
        self.right = right

class UnaryOp:
    def __init__(self, op, operand):
        self.op = op
        self.operand = operand

class FuncCall:
    def __init__(self, name, args):
        self.name = name
        self.args = args

class VarDecl:
    def __init__(self, var_type, name, value):
        self.var_type = var_type
        self.name = name
        self.value = value

class ArrayDecl:
    def __init__(self, var_type, name, size):
        self.var_type = var_type
        self.name = name
        self.size = size

class ArrayAccess:
    def __init__(self, name, index):
        self.name = name
        self.index = index

class Assign:
    def __init__(self, target, value):
        self.target = target
        self.value = value

class If:
    def __init__(self, condition, then_body, else_body=None):
        self.condition = condition
        self.then_body = then_body
        self.else_body = else_body

class While:
    def __init__(self, condition, body):
        self.condition = condition
        self.body = body

class For:
    def __init__(self, init, condition, update, body):
        self.init = init
        self.condition = condition
        self.update = update
        self.body = body

class FunctionDef:
    def __init__(self, name, params, return_type, body):
        self.name = name
        self.params = params
        self.return_type = return_type
        self.body = body

class Return:
    def __init__(self, value):
        self.value = value

class Import:
    def __init__(self, module):
        self.module = module

class Program:
    def __init__(self, stmts):
        self.stmts = stmts

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

class FuncCall:
    def __init__(self, name, args):
        self.name = name
        self.args = args

class VarDecl:
    def __init__(self, var_type, name, value):
        self.var_type = var_type
        self.name = name
        self.value = value

class Assign:
    def __init__(self, name, value):
        self.name = name
        self.value = value

class Program:
    def __init__(self, stmts):
        self.stmts = stmts

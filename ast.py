class Number:
    def __init__(self, value):
        self.value = value

class BinOp:
    def __init__(self, left, op, right):
        self.left = left
        self.op = op
        self.right = right

class Print:
    def __init__(self, expr):
        self.expr = expr

class Program:
    def __init__(self, stmts):
        self.stmts = stmts

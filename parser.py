import ply.yacc as yacc

from lexer import tokens
from nodes import Number, BinOp, Program, FuncCall, String, VarDecl, Assign, Identifier, If, While, For, FunctionDef, Return, Import, UnaryOp

def p_program(p):
    'program : statement_list'
    p[0] = Program(p[1])

def p_statement_list(p):
    '''statement_list : statement_list statement
                      | statement'''
    if len(p) == 2:
        p[0] = [p[1]]
    else:
        p[0] = p[1] + [p[2]]

def p_statement(p):
    '''statement : var_declaration
                 | assignment
                 | expression SEMI
                 | if_statement
                 | while_statement
                 | for_statement
                 | return_statement
                 | function_definition
                 | import_statement'''
    p[0] = p[1]

def p_import_statement(p):
    'import_statement : IMPORT ID'
    p[0] = Import(p[2])
def p_function_definition(p):
    'function_definition : DEF ID LPAREN parameters RPAREN ARROW type block'
    p[0] = FunctionDef(p[2], p[4], p[7], p[8])

def p_parameters(p):
    '''parameters : parameters COMMA parameter
                  | parameter
                  |'''
    if len(p) == 1:
        p[0] = []
    elif len(p) == 2:
        p[0] = [p[1]]
    else:
        p[0] = p[1] + [p[3]]

def p_parameter(p):
    'parameter : type ID'
    p[0] = (p[1], p[2])

def p_return_statement(p):
    'return_statement : RETURN expression SEMI'
    p[0] = Return(p[2])

def p_block(p):
    'block : LCURLY statement_list RCURLY'
    p[0] = p[2]

def p_if_statement(p):
    '''if_statement : IF LPAREN expression RPAREN block
                    | IF LPAREN expression RPAREN block ELSE block'''
    if len(p) == 6:
        p[0] = If(p[3], p[5])
    else:
        p[0] = If(p[3], p[5], p[7])

def p_while_statement(p):
    'while_statement : WHILE LPAREN expression RPAREN block'
    p[0] = While(p[3], p[5])

def p_for_statement(p):
    'for_statement : FOR LPAREN for_init expression SEMI assignment_no_semi RPAREN block'
    p[0] = For(p[3], p[4], p[6], p[8])

def p_for_init(p):
    '''for_init : assignment
                | var_declaration'''
    p[0] = p[1]

def p_assignment_no_semi(p):
    'assignment_no_semi : expression EQUALS expression'
    p[0] = Assign(p[1], p[3])

def p_var_declaration(p):
    'var_declaration : type ID EQUALS expression SEMI'
    p[0] = VarDecl(p[1], p[2], p[4])

def p_type(p):
    '''type : INT
            | FLOAT
            | CHAR
            | type TIMES'''
    if len(p) == 2:
        p[0] = p[1]
    else:
        p[0] = p[1] + '*'

def p_assignment(p):
    'assignment : expression EQUALS expression SEMI'
    p[0] = Assign(p[1], p[3])

def p_expression_func_call(p):
    'expression : func_name LPAREN arg_list RPAREN'
    p[0] = FuncCall(p[1], p[3])

def p_func_name(p):
    '''func_name : PRINT
                 | FOPEN
                 | FREAD
                 | FWRITE
                 | FCLOSE
                 | ID'''
    p[0] = p[1]

def p_arg_list(p):
    '''arg_list : arg_list COMMA expression
                | expression
                |'''
    if len(p) == 1:
        p[0] = []
    elif len(p) == 2:
        p[0] = [p[1]]
    else:
        p[0] = p[1] + [p[3]]

precedence = (
    ('left', 'EQ', 'NOT_EQ'),
    ('left', 'LESS', 'GREATER', 'LESS_EQ', 'GREATER_EQ'),
    ('left', 'PLUS', 'MINUS'),
    ('left', 'TIMES', 'DIVIDE'),
    ('right', 'UNARY'),
)

# Parsing rules
def p_expression_binop(p):
    '''expression : expression PLUS expression
                  | expression MINUS expression
                  | expression TIMES expression
                  | expression DIVIDE expression
                  | expression LESS expression
                  | expression GREATER expression
                  | expression LESS_EQ expression
                  | expression GREATER_EQ expression
                  | expression EQ expression
                  | expression NOT_EQ expression'''
    p[0] = BinOp(p[1], p[2], p[3])

def p_expression_group(p):
    'expression : LPAREN expression RPAREN'
    p[0] = p[2]

def p_expression_number(p):
    'expression : NUMBER'
    p[0] = Number(p[1])

def p_expression_string(p):
    'expression : STRING'
    p[0] = String(p[1])

def p_expression_unaryop(p):
    '''expression : AMPERSAND expression %prec UNARY
                  | TIMES expression %prec UNARY'''
    p[0] = UnaryOp(p[1], p[2])

def p_expression_id(p):
    'expression : ID'
    p[0] = Identifier(p[1])

# Error rule for syntax errors
def p_error(p):
    if p:
        print(f"Syntax error at '{p.value}', line {p.lineno}")
    else:
        print("Syntax error at EOF")

# Build the parser
parser = yacc.yacc(start='program')

import ply.yacc as yacc

from lexer import tokens
from ast import Number, BinOp, Program, FuncCall, String, VarDecl, Assign, Identifier

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
                 | expression SEMI'''
    p[0] = p[1]

def p_var_declaration(p):
    'var_declaration : type ID EQUALS expression SEMI'
    p[0] = VarDecl(p[1], p[2], p[4])

def p_type(p):
    '''type : INT
            | FLOAT
            | CHAR'''
    p[0] = p[1]

def p_assignment(p):
    'assignment : ID EQUALS expression SEMI'
    p[0] = Assign(p[1], p[3])

def p_expression_func_call(p):
    'expression : func_name LPAREN arg_list RPAREN'
    p[0] = FuncCall(p[1], p[3])

def p_func_name(p):
    '''func_name : PRINT
                 | FOPEN
                 | FREAD
                 | FWRITE
                 | FCLOSE'''
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

# Parsing rules
def p_expression_binop(p):
    '''expression : expression PLUS expression
                  | expression MINUS expression
                  | expression TIMES expression
                  | expression DIVIDE expression'''
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

def p_expression_id(p):
    'expression : ID'
    p[0] = Identifier(p[1])

# Error rule for syntax errors
def p_error(p):
    print("Syntax error in input!")

# Build the parser
parser = yacc.yacc(start='program')

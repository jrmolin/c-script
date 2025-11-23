import ply.lex as lex
import codecs

# List of token names.
tokens = (
    'ID',
    'NUMBER',
    'PLUS',
    'MINUS',
    'TIMES',
    'DIVIDE',
    'LPAREN',
    'RPAREN',
    'LCURLY',
    'RCURLY',
    'PRINT',
    'SEMI',
    'STRING',
    'FOPEN',
    'FREAD',
    'FWRITE',
    'FCLOSE',
    'COMMA',
    'INT',
    'FLOAT',
    'CHAR',
    'IF',
    'ELSE',
    'WHILE',
    'FOR',
    'EQUALS',
    'LESS',
    'GREATER',
    'LESS_EQ',
    'GREATER_EQ',
    'NOT_EQ',
    'EQ',
)

# Regular expression rules for simple tokens
t_PLUS    = r'\+'
t_MINUS   = r'-'
t_TIMES   = r'\*'
t_DIVIDE  = r'/'
t_LPAREN  = r'\('
t_RPAREN  = r'\)'
t_LCURLY  = r'\{'
t_RCURLY  = r'\}'
t_SEMI    = r';'
t_COMMA   = r','
t_EQUALS  = r'='
t_LESS    = r'<'
t_GREATER = r'>'
t_LESS_EQ = r'<='
t_GREATER_EQ = r'>='
t_NOT_EQ = r'!='
t_EQ = r'=='


reserved = {
    'print': 'PRINT',
    'fopen': 'FOPEN',
    'fread': 'FREAD',
    'fwrite': 'FWRITE',
    'fclose': 'FCLOSE',
    'int': 'INT',
    'float': 'FLOAT',
    'char': 'CHAR',
    'if': 'IF',
    'else': 'ELSE',
    'while': 'WHILE',
    'for': 'FOR',
}

def t_ID(t):
    r'[a-zA-Z_][a-zA-Z_0-9]*'
    t.type = reserved.get(t.value, 'ID')
    return t

def t_STRING(t):
    r'"[^"]*"'
    t.value = codecs.escape_decode(bytes(t.value[1:-1], "utf-8"))[0].decode("utf-8")
    return t

# A regular expression rule with some action code
def t_NUMBER(t):
    r'\d+'
    t.value = int(t.value)
    return t

# Define a rule so we can track line numbers
def t_newline(t):
    r'\n+'
    t.lexer.lineno += len(t.value)

# A string containing ignored characters (spaces and tabs)
t_ignore  = ' \t'

# Error handling rule
def t_error(t):
    print("Illegal character '%s'" % t.value[0])
    t.lexer.skip(1)

# Build the lexer
lexer = lex.lex()

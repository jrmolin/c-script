#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Char,
    Pointer(Box<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOpType {
    Add,
    Sub,
    Mul,
    Div,
    Less,
    Greater,
    LessEq,
    GreaterEq,
    Eq,
    NotEq,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOpType {
    AddressOf,
    Deref,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Number(i32),
    String(String),
    Identifier(String),
    BinOp(Box<Expression>, BinOpType, Box<Expression>),
    UnaryOp(UnaryOpType, Box<Expression>),
    FuncCall(String, Vec<Expression>),
    ArrayAccess(String, Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VarDecl(Type, String, Option<Expression>),
    ArrayDecl(Type, String, i32),
    Assign(Expression, Expression),
    Expression(Expression),
    If(Expression, Box<Vec<Statement>>, Option<Box<Vec<Statement>>>),
    While(Expression, Box<Vec<Statement>>),
    For(
        Box<Statement>, // Init (Assign or VarDecl)
        Expression,     // Condition
        Box<Statement>, // Update (Assign)
        Box<Vec<Statement>>, // Body
    ),
    Return(Expression),
    FunctionDef(String, Vec<(Type, String)>, Type, Box<Vec<Statement>>),
    Import(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

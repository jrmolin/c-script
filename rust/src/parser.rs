use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace1, not_line_ending},
    combinator::{map, map_res, opt, recognize, value},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded},
    IResult,
};

use crate::ast::*;

// --- Whitespace and Comments ---

fn line_comment<'a, E: nom::error::ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    let (input, _) = tag("//")(input)?;
    let (input, comment) = not_line_ending(input)?;
    Ok((input, comment))
}

fn block_comment<'a, E: nom::error::ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    let (input, _) = tag("/*")(input)?;
    let (input, comment) = take_until("*/")(input)?;
    let (input, _) = tag("*/")(input)?;
    Ok((input, comment))
}

fn ws<'a, F: 'a, O, E: nom::error::ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(
        many0(alt((multispace1, line_comment, block_comment))),
        inner,
        many0(alt((multispace1, line_comment, block_comment))),
    )
}

// --- Identifiers and Keywords ---

fn identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: &str| s.to_string(),
    )(input)
}

// --- Types ---

fn parse_base_type(input: &str) -> IResult<&str, Type> {
    alt((
        value(Type::Int, tag("int")),
        value(Type::Float, tag("float")),
        value(Type::Char, tag("char")),
    ))(input)
}

fn parse_type(input: &str) -> IResult<&str, Type> {
    let (input, mut t) = parse_base_type(input)?;
    let (input, stars) = many0(ws(char('*')))(input)?;
    for _ in stars {
        t = Type::Pointer(Box::new(t));
    }
    Ok((input, t))
}

// --- Expressions ---

fn parse_number(input: &str) -> IResult<&str, Expression> {
    map_res(digit1, |s: &str| s.parse::<i32>().map(Expression::Number))(input)
}

fn parse_string_literal(input: &str) -> IResult<&str, Expression> {
    let (input, s) = delimited(char('"'), take_while(|c| c != '"'), char('"'))(input)?;
    Ok((input, Expression::String(s.to_string())))
}

fn parse_primary_expr(input: &str) -> IResult<&str, Expression> {
    alt((
        parse_number,
        parse_string_literal,
        parse_func_call,
        parse_array_access,
        map(identifier, Expression::Identifier),
        delimited(char('('), ws(parse_expression), char(')')),
    ))(input)
}

fn parse_unary_expr(input: &str) -> IResult<&str, Expression> {
    alt((
        map(preceded(char('&'), ws(parse_unary_expr)), |e| {
            Expression::UnaryOp(UnaryOpType::AddressOf, Box::new(e))
        }),
        map(preceded(char('*'), ws(parse_unary_expr)), |e| {
            Expression::UnaryOp(UnaryOpType::Deref, Box::new(e))
        }),
        parse_primary_expr,
    ))(input)
}

// Helper for binary operations to handle precedence correctly
// For simplicity in this recursive descent example, we'll implement a simple precedence climber or just nested calls.
// Let's use nested calls for standard precedence: Mul/Div > Add/Sub > Relational > Eq/Neq

fn parse_term(input: &str) -> IResult<&str, Expression> {
    let (input, init) = ws(parse_unary_expr)(input)?;
    let (input, res) = many0(pair(
        ws(alt((
            value(BinOpType::Mul, char('*')),
            value(BinOpType::Div, char('/')),
        ))),
        ws(parse_unary_expr),
    ))(input)?;

    Ok((
        input,
        res.into_iter().fold(init, |acc, (op, val)| {
            Expression::BinOp(Box::new(acc), op, Box::new(val))
        }),
    ))
}

fn parse_arith_expr(input: &str) -> IResult<&str, Expression> {
    let (input, init) = ws(parse_term)(input)?;
    let (input, res) = many0(pair(
        ws(alt((
            value(BinOpType::Add, char('+')),
            value(BinOpType::Sub, char('-')),
        ))),
        ws(parse_term),
    ))(input)?;

    Ok((
        input,
        res.into_iter().fold(init, |acc, (op, val)| {
            Expression::BinOp(Box::new(acc), op, Box::new(val))
        }),
    ))
}

fn parse_relational_expr(input: &str) -> IResult<&str, Expression> {
    let (input, init) = ws(parse_arith_expr)(input)?;
    let (input, res) = many0(pair(
        ws(alt((
            value(BinOpType::LessEq, tag("<=")),
            value(BinOpType::GreaterEq, tag(">=")),
            value(BinOpType::Less, char('<')),
            value(BinOpType::Greater, char('>')),
        ))),
        ws(parse_arith_expr),
    ))(input)?;

    Ok((
        input,
        res.into_iter().fold(init, |acc, (op, val)| {
            Expression::BinOp(Box::new(acc), op, Box::new(val))
        }),
    ))
}

fn parse_expression(input: &str) -> IResult<&str, Expression> {
    let (input, init) = ws(parse_relational_expr)(input)?;
    let (input, res) = many0(pair(
        ws(alt((
            value(BinOpType::Eq, tag("==")),
            value(BinOpType::NotEq, tag("!=")),
        ))),
        ws(parse_relational_expr),
    ))(input)?;

    Ok((
        input,
        res.into_iter().fold(init, |acc, (op, val)| {
            Expression::BinOp(Box::new(acc), op, Box::new(val))
        }),
    ))
}

fn parse_func_call(input: &str) -> IResult<&str, Expression> {
    let (input, id) = identifier(input)?;
    let (input, args) = delimited(
        char('('),
        ws(separated_list0(char(','), ws(parse_expression))),
        char(')'),
    )(input)?;
    Ok((input, Expression::FuncCall(id, args)))
}

fn parse_array_access(input: &str) -> IResult<&str, Expression> {
    let (input, id) = identifier(input)?;
    let (input, index) = delimited(char('['), ws(parse_expression), char(']'))(input)?;
    Ok((input, Expression::ArrayAccess(id, Box::new(index))))
}

// --- Statements ---

fn parse_var_decl(input: &str) -> IResult<&str, Statement> {
    let (input, ty) = ws(parse_type)(input)?;
    let (input, id) = ws(identifier)(input)?;

    // Check for array declaration
    if let Ok((input, _)) = ws(char::<&str, nom::error::Error<&str>>('['))(input) {
        let (input, size) = ws(digit1)(input)?;
        let (input, _) = ws(char(']'))(input)?;
        let (input, _) = ws(char(';'))(input)?;
        return Ok((input, Statement::ArrayDecl(ty, id, size.parse().unwrap())));
    }

    let (input, init) = opt(preceded(ws(char('=')), ws(parse_expression)))(input)?;
    let (input, _) = ws(char(';'))(input)?;
    Ok((input, Statement::VarDecl(ty, id, init)))
}

fn parse_assign(input: &str) -> IResult<&str, Statement> {
    let (input, lhs) = ws(parse_unary_expr)(input)?; // Changed from parse_primary_expr to parse_unary_expr
    let (input, _) = ws(char('='))(input)?;
    let (input, rhs) = ws(parse_expression)(input)?;
    let (input, _) = ws(char(';'))(input)?;
    Ok((input, Statement::Assign(lhs, rhs)))
}

fn parse_assign_no_semi(input: &str) -> IResult<&str, Statement> {
    let (input, lhs) = ws(parse_unary_expr)(input)?; // Changed from parse_primary_expr to parse_unary_expr
    let (input, _) = ws(char('='))(input)?;
    let (input, rhs) = ws(parse_expression)(input)?;
    Ok((input, Statement::Assign(lhs, rhs)))
}

fn parse_expr_stmt(input: &str) -> IResult<&str, Statement> {
    let (input, expr) = ws(parse_expression)(input)?;
    let (input, _) = ws(char(';'))(input)?;
    Ok((input, Statement::Expression(expr)))
}

fn parse_block(input: &str) -> IResult<&str, Vec<Statement>> {
    delimited(ws(char('{')), many0(ws(parse_statement)), ws(char('}')))(input)
}

fn parse_if(input: &str) -> IResult<&str, Statement> {
    let (input, _) = ws(tag("if"))(input)?;
    let (input, cond) = delimited(char('('), ws(parse_expression), char(')'))(input)?;
    let (input, then_block) = ws(parse_block)(input)?;
    let (input, else_block) = opt(preceded(ws(tag("else")), ws(parse_block)))(input)?;
    Ok((
        input,
        Statement::If(cond, Box::new(then_block), else_block.map(Box::new)),
    ))
}

fn parse_while(input: &str) -> IResult<&str, Statement> {
    let (input, _) = ws(tag("while"))(input)?;
    let (input, cond) = delimited(char('('), ws(parse_expression), char(')'))(input)?;
    let (input, body) = ws(parse_block)(input)?;
    Ok((input, Statement::While(cond, Box::new(body))))
}

fn parse_for(input: &str) -> IResult<&str, Statement> {
    let (input, _) = ws(tag("for"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    // Init can be var decl or assignment
    let (input, init) = alt((
        parse_var_decl,
        parse_assign, // parse_assign consumes the semicolon
    ))(input)?;

    let (input, cond) = ws(parse_expression)(input)?;
    let (input, _) = ws(char(';'))(input)?;

    let (input, update) = ws(parse_assign_no_semi)(input)?;
    let (input, _) = ws(char(')'))(input)?;

    let (input, body) = ws(parse_block)(input)?;

    Ok((
        input,
        Statement::For(Box::new(init), cond, Box::new(update), Box::new(body)),
    ))
}

fn parse_return(input: &str) -> IResult<&str, Statement> {
    let (input, _) = ws(tag("return"))(input)?;
    let (input, expr) = ws(parse_expression)(input)?;
    let (input, _) = ws(char(';'))(input)?;
    Ok((input, Statement::Return(expr)))
}

fn parse_import(input: &str) -> IResult<&str, Statement> {
    let (input, _) = ws(tag("import"))(input)?;
    let (input, id) = ws(identifier)(input)?;
    Ok((input, Statement::Import(id)))
}

fn parse_func_def(input: &str) -> IResult<&str, Statement> {
    let (input, _) = ws(tag("def"))(input)?;
    let (input, name) = ws(identifier)(input)?;
    let (input, params) = delimited(
        char('('),
        separated_list0(ws(char(',')), pair(ws(parse_type), ws(identifier))),
        char(')'),
    )(input)?;

    let (input, _) = ws(tag("->"))(input)?;
    let (input, ret_type) = ws(parse_type)(input)?;
    let (input, body) = ws(parse_block)(input)?;

    Ok((
        input,
        Statement::FunctionDef(name, params, ret_type, Box::new(body)),
    ))
}

fn parse_statement(input: &str) -> IResult<&str, Statement> {
    alt((
        parse_import,
        parse_func_def,
        parse_if,
        parse_while,
        parse_for,
        parse_return,
        parse_var_decl,
        parse_assign,
        parse_expr_stmt,
    ))(input)
}

pub fn parse_program(input: &str) -> IResult<&str, Program> {
    let (input, statements) = many0(ws(parse_statement))(input)?;
    Ok((input, Program { statements }))
}

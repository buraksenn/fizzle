use std::collections::HashMap;

use crate::{
    ast::{
        BlockStatement, CallExpression, Expression, FunctionExpression, HashExpression,
        IfExpression, IndexExpression, Node, Program, Statement,
    },
    lexer::Lexer,
    parser::precedence::Precedence,
    token::Token,
};
use anyhow::anyhow;
use log::debug;

mod precedence;

type ParserResult<T> = anyhow::Result<T>;
type PrefixParseFn = fn(&mut Parser<'_>) -> ParserResult<Expression>;
type InfixParseFn = fn(&mut Parser<'_>, Expression) -> ParserResult<Expression>;

pub fn parse(input: &str) -> ParserResult<Node> {
    let mut parser = Parser::new(Lexer::new(input));
    let prog = parser.parse_program()?;
    Ok(Node::Program(Box::new(prog)))
}

pub struct Parser<'a> {
    l: Lexer<'a>,

    current_token: Token,
    peek_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(mut l: Lexer<'a>) -> Self {
        let current = l.next_token();
        let next = l.next_token();

        Self {
            l,
            current_token: current,
            peek_token: next,
        }
    }

    fn next_token(&mut self) {
        self.current_token = std::mem::replace(&mut self.peek_token, self.l.next_token());
    }

    pub fn parse_program(&mut self) -> ParserResult<Program> {
        let mut program = Program::default();
        let mut parse_error: Option<anyhow::Error> = None;

        while self.current_token != Token::Eof {
            match self.parse_statement() {
                Ok(st) => program.statements.push(st),
                Err(e) => match parse_error.take() {
                    Some(old) => parse_error = Some(old.context(e)),
                    None => parse_error = Some(e),
                },
            };
            self.next_token();
        }

        if let Some(err) = parse_error {
            return Err(err);
        }
        Ok(program)
    }

    fn parse_statement(&mut self) -> ParserResult<Statement> {
        match self.current_token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_expression_statement(&mut self) -> ParserResult<Statement> {
        let exp = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token_is(&Token::Semicolon) {
            self.next_token();
        }

        Ok(Statement::Expression { value: exp })
    }

    fn parse_expression(&mut self, precedence: Precedence) -> ParserResult<Expression> {
        let mut left_exp: Expression;

        if let Some(prefix) = self.current_prefix_fn() {
            left_exp = prefix(self)?;
        } else {
            return Err(anyhow!(
                "could not find prefix function for token: {}",
                self.current_token
            ));
        }

        while !self.peek_token_is(&Token::Semicolon) && precedence < self.peek_precendence() {
            if let Some(infix) = self.peek_infix_fn() {
                self.next_token();
                left_exp = infix(self, left_exp)?;
            } else {
                return Ok(left_exp);
            }
        }

        Ok(left_exp)
    }

    fn peek_infix_fn(&mut self) -> Option<InfixParseFn> {
        match self.peek_token {
            Token::Plus
            | Token::Minus
            | Token::Slash
            | Token::Asterisk
            | Token::Eq
            | Token::Neq
            | Token::Lt
            | Token::Gt => Some(parse_infix_expression),
            Token::Lparen => Some(parse_call_expression),
            Token::Lbracket => Some(parse_index_expression),
            _ => None,
        }
    }

    fn current_prefix_fn(&mut self) -> Option<PrefixParseFn> {
        match self.current_token {
            Token::Ident(_) => Some(parse_identifier),
            Token::String(_) => Some(parse_string_literal),
            Token::Int(_) => Some(parse_integer_literal),
            Token::True | Token::False => Some(parse_boolean_expression),
            Token::Minus | Token::Bang => Some(parse_prefix_expression),
            Token::Lparen => Some(parse_grouped_expression),
            Token::If => Some(parse_if_expression),
            Token::Function => Some(parse_function_expression),
            Token::Lbracket => Some(parse_array_literal),
            Token::Lbrace => Some(parse_hash_expression),
            _ => None,
        }
    }

    fn parse_let_statement(&mut self) -> ParserResult<Statement> {
        let name = self.expect_ident()?;
        self.expect_peek(Token::Assign)?;
        self.next_token();

        let exp = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token_is(&Token::Semicolon) {
            self.next_token();
        }

        Ok(Statement::Let { name, value: exp })
    }

    fn parse_return_statement(&mut self) -> ParserResult<Statement> {
        self.next_token();
        let value = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token_is(&Token::Semicolon) {
            self.next_token();
        }

        Ok(Statement::Return { value })
    }

    fn parse_expression_list(&mut self, end: Token) -> ParserResult<Vec<Expression>> {
        let mut list: Vec<Expression> = Vec::new();

        if self.peek_token_is(&end) {
            self.next_token();
            return Ok(list);
        }

        self.next_token();
        list.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token_is(&Token::Comma) {
            self.next_token();
            self.next_token();
            list.push(self.parse_expression(Precedence::Lowest)?);
        }

        self.expect_peek(end)?;

        Ok(list)
    }

    fn expect_peek(&mut self, tok: Token) -> ParserResult<()> {
        if self.peek_token_is(&tok) {
            self.next_token();
            return Ok(());
        } else {
            Err(anyhow!(
                "expected token {} but received: {}",
                tok,
                self.peek_token
            ))
        }
    }

    fn expect_ident(&mut self) -> ParserResult<String> {
        let name = match &self.peek_token {
            Token::Ident(name) => name.to_string(),
            tok => return Err(anyhow!("expected ident token but received: {}", tok)),
        };
        self.next_token();

        Ok(name)
    }

    fn expect_current_token_is(&self, tok: Token) -> bool {
        match (&tok, &self.current_token) {
            (Token::Ident(_), Token::Ident(_)) => true,
            (Token::Int(_), Token::Int(_)) => true,
            _ => tok == self.current_token,
        }
    }

    fn peek_token_is(&self, tok: &Token) -> bool {
        match (&tok, &self.peek_token) {
            (Token::Ident(_), Token::Ident(_)) => true,
            (Token::Int(_), Token::Int(_)) => true,
            _ => tok == &self.peek_token,
        }
    }

    fn peek_precendence(&self) -> Precedence {
        Precedence::from_token(&self.peek_token)
    }

    fn current_precendence(&self) -> Precedence {
        Precedence::from_token(&self.current_token)
    }
}

fn parse_string_literal(parser: &mut Parser<'_>) -> ParserResult<Expression> {
    if let Token::String(ref s) = parser.current_token {
        Ok(Expression::StringLiteral(s.clone()))
    } else {
        Err(anyhow!(
            "expected string literal token but got: {}",
            parser.current_token
        ))
    }
}

fn parse_identifier(parser: &mut Parser<'_>) -> ParserResult<Expression> {
    if let Token::Ident(ref s) = parser.current_token {
        Ok(Expression::Identifier(s.clone()))
    } else {
        Err(anyhow!(
            "expected ident token but got: {}",
            parser.current_token
        ))
    }
}

fn get_string_from_token(tok: &Token) -> ParserResult<String> {
    if let &Token::Ident(ref s) = tok {
        Ok(s.clone())
    } else {
        Err(anyhow!("expected ident token but got: {}", tok))
    }
}

fn parse_integer_literal(parser: &mut Parser<'_>) -> ParserResult<Expression> {
    if let Token::Int(i) = parser.current_token {
        Ok(Expression::IntegerLiteral(i))
    } else {
        Err(anyhow!(
            "expected ident token but got: {}",
            parser.current_token
        ))
    }
}

fn parse_grouped_expression(parser: &mut Parser<'_>) -> ParserResult<Expression> {
    parser.next_token();

    let exp = parser.parse_expression(Precedence::Lowest)?;

    parser.expect_peek(Token::Rparen)?;

    Ok(exp)
}

fn parse_call_expression(parser: &mut Parser<'_>, left: Expression) -> ParserResult<Expression> {
    debug!(
        "Entered call expression current token: {}, peek token: {}",
        parser.current_token, parser.peek_token
    );
    Ok(Expression::Call(Box::new(CallExpression {
        function: left,
        arguments: parser.parse_expression_list(Token::Rparen)?,
    })))
}

fn parse_index_expression(parser: &mut Parser<'_>, left: Expression) -> ParserResult<Expression> {
    parser.next_token();
    let exp = Expression::Index(Box::new(IndexExpression {
        left: left,
        index: parser.parse_expression(Precedence::Lowest)?,
    }));
    parser.expect_peek(Token::Rbracket)?;

    Ok(exp)
}

fn parse_function_parameters(parser: &mut Parser<'_>) -> ParserResult<Vec<String>> {
    let mut parameters: Vec<String> = Vec::new();
    if parser.peek_token_is(&Token::Rparen) {
        parser.next_token();
        return Ok(parameters);
    }
    parser.next_token();

    parameters.push(get_string_from_token(&parser.current_token)?);

    while parser.peek_token_is(&Token::Comma) {
        parser.next_token();
        parser.next_token();
        parameters.push(get_string_from_token(&parser.current_token)?);
    }
    parser.expect_peek(Token::Rparen)?;

    Ok(parameters)
}

fn parse_function_expression(parser: &mut Parser<'_>) -> ParserResult<Expression> {
    parser.expect_peek(Token::Lparen)?;

    let parameters = parse_function_parameters(parser)?;
    parser.expect_peek(Token::Lbrace)?;

    // skip lbrace
    parser.next_token();
    let body = parse_block_statement(parser)?;

    Ok(Expression::Function(Box::new(FunctionExpression {
        parameters,
        body,
    })))
}

// TODO: add capability for `else if`
fn parse_if_expression(parser: &mut Parser<'_>) -> ParserResult<Expression> {
    parser.expect_peek(Token::Lparen)?;
    // skip lparen
    parser.next_token();

    let condition = parser.parse_expression(Precedence::Lowest)?;
    parser.expect_peek(Token::Rparen)?;
    parser.expect_peek(Token::Lbrace)?;

    // skip lbrace
    parser.next_token();
    let consequence = parse_block_statement(parser)?;

    let alternative = if parser.peek_token_is(&Token::Else) {
        // skip else
        parser.next_token();
        parser.expect_peek(Token::Lbrace)?;

        // skip lbrace
        parser.next_token();
        Some(parse_block_statement(parser)?)
    } else {
        None
    };

    Ok(Expression::If(Box::new(IfExpression {
        condition,
        consequence,
        alternative,
    })))
}

fn parse_block_statement(parser: &mut Parser<'_>) -> ParserResult<BlockStatement> {
    let mut statements: Vec<Statement> = Vec::new();

    while !parser.expect_current_token_is(Token::Eof)
        && !parser.expect_current_token_is(Token::Rbrace)
    {
        let st = parser.parse_statement()?;
        parser.next_token();
        statements.push(st);
    }

    Ok(BlockStatement { statements })
}

fn parse_boolean_expression(parser: &mut Parser<'_>) -> ParserResult<Expression> {
    match parser.current_token {
        Token::True => Ok(Expression::Boolean(true)),
        Token::False => Ok(Expression::Boolean(false)),
        _ => Err(anyhow!(
            "expected boolean token but got: {}",
            parser.current_token
        )),
    }
}

fn parse_array_literal(parser: &mut Parser<'_>) -> ParserResult<Expression> {
    Ok(Expression::Array(
        parser.parse_expression_list(Token::Rbracket)?,
    ))
}

fn parse_hash_expression(parser: &mut Parser<'_>) -> ParserResult<Expression> {
    let mut map = HashMap::new();

    if parser.peek_token_is(&Token::Rbrace) {
        parser.next_token();
        return Ok(Expression::HashMap(Box::new(HashExpression { map })));
    }

    loop {
        parser.next_token();
        let key = parser.parse_expression(Precedence::Lowest)?;
        parser.expect_peek(Token::Colon)?;
        parser.next_token();
        let val = parser.parse_expression(Precedence::Lowest)?;
        map.insert(key, val);

        if !parser.peek_token_is(&Token::Comma) {
            break;
        }
        parser.next_token();
    }

    parser.expect_peek(Token::Rbrace)?;

    Ok(Expression::HashMap(Box::new(HashExpression { map })))
}

fn parse_prefix_expression(parser: &mut Parser<'_>) -> ParserResult<Expression> {
    let tok = parser.current_token.clone();
    parser.next_token();
    let exp = parser.parse_expression(Precedence::Prefix)?;

    Ok(Expression::Prefix {
        operator: tok,
        operand: Box::new(exp),
    })
}

fn parse_infix_expression(parser: &mut Parser<'_>, left: Expression) -> ParserResult<Expression> {
    let tok = parser.current_token.clone();
    let precedence = parser.current_precendence();

    parser.next_token();

    let right = parser.parse_expression(precedence)?;

    Ok(Expression::Infix {
        left: Box::new(left),
        operator: tok,
        right: Box::new(right),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ast::Statement, lexer::Lexer};

    #[test]
    fn if_expression() {
        let input = "if (x < y) { x }";

        let prog = setup(input, 1);
        let exp = unwrap_first_expression_from_prog(&prog);

        match exp {
            Expression::If(ifexpr) => {
                test_if_condition(&ifexpr.condition, Token::Lt, "x", "y");

                assert_eq!(
                    ifexpr.consequence.statements.len(),
                    1,
                    "expected only 1 statement"
                );
                match ifexpr.consequence.statements.first().unwrap() {
                    Statement::Expression { value } => test_identifier(value, "x"),
                    stmt => panic!("expected expression statement but got {:?}", stmt),
                }
                if let Some(stmt) = &ifexpr.alternative {
                    panic!("expected alternative to be None but got {:?}", stmt)
                }
            }
            _ => panic!("expected if expression but got {:?}", exp),
        }
    }

    #[test]
    fn if_else_expression() {
        let input = "if (x < y) { x } else { y }";

        let prog = setup(input, 1);
        let exp = unwrap_first_expression_from_prog(&prog);

        match exp {
            Expression::If(ifexpr) => {
                test_if_condition(&ifexpr.condition, Token::Lt, "x", "y");

                assert_eq!(ifexpr.consequence.statements.len(), 1);
                match &ifexpr.consequence.statements.first().unwrap() {
                    Statement::Expression { value } => test_identifier(value, "x"),
                    stmt => panic!("expected expression statement but got {:?}", stmt),
                }

                if let Some(stmt) = &ifexpr.alternative {
                    assert_eq!(stmt.statements.len(), 1);
                    match stmt.statements.first().unwrap() {
                        Statement::Expression { value } => test_identifier(value, "y"),
                        stmt => panic!("expected expression statement but got {:?}", stmt),
                    }
                } else {
                    panic!("expected alternative block")
                }
            }
            _ => panic!("expected if expression but got {:?}", exp),
        }
    }

    #[test]
    fn test_let_statement() {
        let prog = setup(
            "\
            let x = 5;
            let y = 10;
            let foobar = 838383;",
            3,
        );

        let identifiers = vec!["x", "y", "foobar"];
        let integers = vec![5, 10, 838383];

        let mut itr = prog.statements.iter();

        for (t, i) in identifiers.iter().zip(integers) {
            match itr.next().unwrap() {
                Statement::Let { name, value } => {
                    assert_eq!(name, t);
                    test_integer_literal(value, i);
                }
                _ => panic!("unknown node"),
            }
        }
    }

    #[test]
    fn test_return_statement() {
        let prog = setup(
            "\
            return 5;
            return 10;
            return add(3,5);",
            3,
        );

        assert_eq!(prog.statements.len(), 3);

        // Check first return statement: return 5;
        match &prog.statements[0] {
            Statement::Return { value } => match value {
                Expression::IntegerLiteral(val) => assert_eq!(*val, 5),
                _ => panic!("expected IntegerLiteral(5)"),
            },
            _ => panic!("expected Return statement"),
        }

        // Check second return statement: return 10;
        match &prog.statements[1] {
            Statement::Return { value } => match value {
                Expression::IntegerLiteral(val) => assert_eq!(*val, 10),
                _ => panic!("expected IntegerLiteral(10)"),
            },
            _ => panic!("expected Return statement"),
        }

        // Check third return statement: return add(3,5);
        match &prog.statements[2] {
            Statement::Return { value } => {
                match value {
                    Expression::Call(call_expr) => {
                        // Check function name
                        match &call_expr.function {
                            Expression::Identifier(name) => assert_eq!(name, "add"),
                            _ => panic!("expected Identifier 'add'"),
                        }
                        // Check arguments
                        assert_eq!(call_expr.arguments.len(), 2);
                        match &call_expr.arguments[0] {
                            Expression::IntegerLiteral(val) => assert_eq!(*val, 3),
                            _ => panic!("expected IntegerLiteral(3)"),
                        }
                        match &call_expr.arguments[1] {
                            Expression::IntegerLiteral(val) => assert_eq!(*val, 5),
                            _ => panic!("expected IntegerLiteral(5)"),
                        }
                    }
                    _ => panic!("expected Call expression"),
                }
            }
            _ => panic!("expected Return statement"),
        }
    }

    #[test]
    fn test_identifier_expression() {
        let prog = setup("foobar;", 1);

        match &prog.statements[0] {
            Statement::Expression { value } => match value {
                Expression::Identifier(ident) => {
                    assert_eq!(ident, "foobar", "ident.value not foobar. got={}", ident);
                }
                _ => panic!("exp not Expression::Identifier. got={}", value),
            },
            _ => panic!(
                "program.statements[0] is not Statement::Expression. got={}",
                prog.statements[0]
            ),
        }
    }

    #[test]
    fn test_integer_literals() {
        let prog = setup("5;", 1);

        match &prog.statements[0] {
            Statement::Expression { value } => match value {
                Expression::IntegerLiteral(i) => {
                    assert_eq!(*i, 5, "integer.value not foobar. got={}", i);
                }
                _ => panic!("exp not Expression::Identifier. got={}", value),
            },
            _ => panic!(
                "program.statements[0] is not Statement::Expression. got={}",
                prog.statements[0]
            ),
        }
    }

    #[test]
    fn test_prefix_expressions() {
        struct Test<'a> {
            input: &'a str,
            operator: Token,
            value: i64,
        }

        let tests = vec![
            Test {
                input: "!5;",
                operator: Token::Bang,
                value: 5,
            },
            Test {
                input: "-15;",
                operator: Token::Minus,
                value: 15,
            },
        ];

        for t in tests {
            let prog = setup(t.input, 1);
            let exp = unwrap_first_expression_from_prog(&prog);

            match exp {
                Expression::Prefix { operator, operand } => {
                    assert_eq!(
                        t.operator, *operator,
                        "expected {:?} operator but got {:?}",
                        t.operator, operator
                    );
                    test_integer_literal(operand.as_ref(), t.value);
                }
                e => panic!("expected prefix expression but got {:?}", e),
            }
        }
    }

    #[test]
    fn test_infix_expressions_integer() {
        struct Test<'a> {
            input: &'a str,
            left_value: i64,
            operator: Token,
            right_value: i64,
        }

        let tests = vec![
            Test {
                input: "5 + 5;",
                left_value: 5,
                operator: Token::Plus,
                right_value: 5,
            },
            Test {
                input: "5 - 5;",
                left_value: 5,
                operator: Token::Minus,
                right_value: 5,
            },
            Test {
                input: "5 * 5;",
                left_value: 5,
                operator: Token::Asterisk,
                right_value: 5,
            },
            Test {
                input: "5 / 5;",
                left_value: 5,
                operator: Token::Slash,
                right_value: 5,
            },
            Test {
                input: "5 > 5;",
                left_value: 5,
                operator: Token::Gt,
                right_value: 5,
            },
            Test {
                input: "5 < 5;",
                left_value: 5,
                operator: Token::Lt,
                right_value: 5,
            },
            Test {
                input: "5 == 5;",
                left_value: 5,
                operator: Token::Eq,
                right_value: 5,
            },
            Test {
                input: "5 != 5;",
                left_value: 5,
                operator: Token::Neq,
                right_value: 5,
            },
        ];

        for t in tests {
            let prog = setup(t.input, 1);
            let exp = unwrap_first_expression_from_prog(&prog);

            match exp {
                Expression::Infix {
                    left,
                    operator,
                    right,
                } => {
                    assert_eq!(
                        t.operator, *operator,
                        "expected {} operator but got {}",
                        t.operator, operator
                    );
                    test_integer_literal(left, t.left_value);
                    test_integer_literal(right, t.right_value);
                }
                exp => panic!("expected prefix expression but got {:?}", exp),
            }
        }
    }

    #[test]
    fn test_infix_expressions_boolean() {
        struct Test<'a> {
            input: &'a str,
            left_value: bool,
            operator: Token,
            right_value: bool,
        }

        let tests = vec![
            Test {
                input: "true == true",
                left_value: true,
                operator: Token::Eq,
                right_value: true,
            },
            Test {
                input: "true != false",
                left_value: true,
                operator: Token::Neq,
                right_value: false,
            },
            Test {
                input: "false == false",
                left_value: false,
                operator: Token::Eq,
                right_value: false,
            },
        ];

        for t in tests {
            let prog = setup(t.input, 1);
            let exp = unwrap_first_expression_from_prog(&prog);

            match exp {
                Expression::Infix {
                    left,
                    operator,
                    right,
                } => {
                    assert_eq!(
                        t.operator, *operator,
                        "expected {} operator but got {}",
                        t.operator, operator
                    );
                    test_boolean_literal(left, t.left_value);
                    test_boolean_literal(right, t.right_value);
                }
                exp => panic!("expected prefix expression but got {:?}", exp),
            }
        }
    }

    #[test]
    fn test_operator_precedence() {
        struct Test<'a> {
            input: &'a str,
            expected: &'a str,
        }

        let tests = vec![
            Test {
                input: "-a * b",
                expected: "((-a) * b)",
            },
            Test {
                input: "!-a",
                expected: "(!(-a))",
            },
            Test {
                input: "a + b + c",
                expected: "((a + b) + c)",
            },
            Test {
                input: "a + b - c",
                expected: "((a + b) - c)",
            },
            Test {
                input: "a * b * c",
                expected: "((a * b) * c)",
            },
            Test {
                input: "a * b / c",
                expected: "((a * b) / c)",
            },
            Test {
                input: "a + b / c",
                expected: "(a + (b / c))",
            },
            Test {
                input: "a + b * c + d / e - f",
                expected: "(((a + (b * c)) + (d / e)) - f)",
            },
            Test {
                input: "3 + 4; -5 * 5",
                expected: "(3 + 4)((-5) * 5)",
            },
            Test {
                input: "5 > 4 == 3 < 4",
                expected: "((5 > 4) == (3 < 4))",
            },
            Test {
                input: "5 < 4 != 3 > 4",
                expected: "((5 < 4) != (3 > 4))",
            },
            Test {
                input: "3 + 4 * 5 == 3 * 1 + 4 * 5",
                expected: "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            },
            Test {
                input: "true",
                expected: "true",
            },
            Test {
                input: "false",
                expected: "false",
            },
            Test {
                input: "3 > 5 == false",
                expected: "((3 > 5) == false)",
            },
            Test {
                input: "3 < 5 == true",
                expected: "((3 < 5) == true)",
            },
            Test {
                input: "1 + (2 + 3) + 4",
                expected: "((1 + (2 + 3)) + 4)",
            },
            Test {
                input: "(5 + 5) * 2",
                expected: "((5 + 5) * 2)",
            },
            Test {
                input: "2 / (5 + 5)",
                expected: "(2 / (5 + 5))",
            },
            Test {
                input: "-(5 + 5)",
                expected: "(-(5 + 5))",
            },
            Test {
                input: "!(true == true)",
                expected: "(!(true == true))",
            },
            Test {
                input: "a + add(b * c) + d",
                expected: "((a + add((b * c))) + d)",
            },
            Test {
                input: "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
                expected: "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))",
            },
            Test {
                input: "add(a + b + c * d / f + g)",
                expected: "add((((a + b) + ((c * d) / f)) + g))",
            },
        ];

        for t in tests {
            let prog = setup(t.input, 0).to_string();

            assert_eq!(
                t.expected, prog,
                "expected '{}' but got '{}'",
                t.expected, prog
            )
        }
    }

    #[test]
    fn test_boolean_expression() {
        struct Test<'a> {
            input: &'a str,
            expected: bool,
        }

        let tests = vec![
            Test {
                input: "true;",
                expected: true,
            },
            Test {
                input: "false;",
                expected: false,
            },
        ];

        for t in tests {
            let prog = setup(t.input, 1);
            let exp = unwrap_first_expression_from_prog(&prog);

            test_boolean_literal(&exp, t.expected);
        }
    }

    #[test]
    fn test_function_literal() {
        let input = "fn(x, y) { x + y; }";
        let prog = setup(input, 1);
        let exp = unwrap_first_expression_from_prog(&prog);

        match exp {
            Expression::Function(func) => {
                assert_eq!(
                    2,
                    func.parameters.len(),
                    "expected 2 parameters but got {:?}",
                    func.parameters
                );
                assert_eq!(func.parameters.first().unwrap(), "x");
                assert_eq!(func.parameters.last().unwrap(), "y");
                assert_eq!(
                    1,
                    func.body.statements.len(),
                    "expecte 1 body statement but got {:?}",
                    func.body.statements
                );

                match func.body.statements.first().unwrap() {
                    Statement::Expression { value } => match value {
                        Expression::Infix {
                            left,
                            operator,
                            right,
                        } => {
                            assert_eq!(*operator, Token::Plus, "expected + but got {}", operator);
                            test_identifier(left.as_ref(), "x");
                            test_identifier(right.as_ref(), "y");
                        }
                        _ => panic!("expected infix expression but got {:?}", value),
                    },
                    stmt => panic!("expected expression statement but got {:?}", stmt),
                }
            }
            _ => panic!("{} is not a function literal", exp),
        }
    }

    #[test]
    fn test_function_parameters() {
        struct Test<'a> {
            input: &'a str,
            expected_params: Vec<&'a str>,
        }

        let tests = vec![
            Test {
                input: "fn() {};",
                expected_params: vec![],
            },
            Test {
                input: "fn(x) {};",
                expected_params: vec!["x"],
            },
            Test {
                input: "fn(x, y, z) {};",
                expected_params: vec!["x", "y", "z"],
            },
        ];

        for t in tests {
            let prog = setup(t.input, 1);
            let exp = unwrap_first_expression_from_prog(&prog);

            match exp {
                Expression::Function(func) => {
                    assert_eq!(func.parameters.len(), t.expected_params.len());
                    let mut params = t.expected_params.into_iter();
                    for param in &func.parameters {
                        let expected_param = params.next().unwrap();
                        assert_eq!(expected_param, param);
                    }
                }
                _ => panic!("{:?} not a function literal", exp),
            }
        }
    }

    #[test]
    fn test_call_expression() {
        let input = "add(1, 2 * 3, 4 + 5);";
        let prog = setup(input, 1);
        let exp = unwrap_first_expression_from_prog(&prog);

        match exp {
            Expression::Call(call) => {
                test_identifier(&call.function, "add");
                assert_eq!(call.arguments.len(), 3);
                let mut args = (&call.arguments).into_iter();
                test_integer_literal(&args.next().unwrap(), 1);
                test_integer_infix(&args.next().unwrap(), 2, Token::Asterisk, 3);
                test_integer_infix(&args.next().unwrap(), 4, Token::Plus, 5)
            }
            _ => panic!("{} is not a call expression", exp),
        }
    }

    fn test_integer_infix(exp: &Expression, l: i64, op: Token, r: i64) {
        match exp {
            Expression::Infix {
                left,
                operator,
                right,
            } => {
                assert_eq!(
                    op, *operator,
                    "expected {} operator but got {}",
                    op, operator
                );
                test_integer_literal(&left, l);
                test_integer_literal(&right, r);
            }
            exp => panic!("expected prefix expression but got {:?}", exp),
        }
    }

    #[test]
    fn call_expression_parameter_parsing() {
        struct Test<'a> {
            input: &'a str,
            expected_ident: &'a str,
            expected_args: Vec<&'a str>,
        }

        let tests = vec![
            Test {
                input: "add();",
                expected_ident: "add",
                expected_args: vec![],
            },
            Test {
                input: "add(1);",
                expected_ident: "add",
                expected_args: vec!["1"],
            },
            Test {
                input: "add(1, 2 * 3, 4 + 5);",
                expected_ident: "add",
                expected_args: vec!["1", "(2 * 3)", "(4 + 5)"],
            },
        ];

        for t in tests {
            let prog = setup(t.input, 1);
            let exp = unwrap_first_expression_from_prog(&prog);

            match exp {
                Expression::Call(call) => {
                    test_identifier(&call.function, t.expected_ident);
                    assert_eq!(call.arguments.len(), t.expected_args.len());
                    let mut args = (&call.arguments).into_iter();
                    for a in t.expected_args {
                        assert_eq!(a.to_string(), args.next().unwrap().to_string());
                    }
                }
                _ => panic!("{:?} is not a call expression", exp),
            }
        }
    }

    #[test]
    fn test_string_literal_expression() {
        let input = r#""hello world""#;
        let prog = setup(input, 1);
        let exp = unwrap_first_expression_from_prog(&prog);

        match exp {
            Expression::StringLiteral(s) => assert_eq!(s, "hello world"),
            _ => panic!("expected string literal but got {:?}", exp),
        }
    }

    #[test]
    fn test_array_literals() {
        let input = "[1, 2 * 2, 3 + 3]";
        let prog = setup(input, 1);
        let exp = unwrap_first_expression_from_prog(&prog);

        match exp {
            Expression::Array(a) => {
                test_integer_literal(a.first().unwrap(), 1);
                test_integer_infix(a.get(1).unwrap(), 2, Token::Asterisk, 2);
                test_integer_infix(a.last().unwrap(), 3, Token::Plus, 3);
            }
            _ => panic!("expected array literal but got {:?}", exp),
        }
    }

    #[test]
    fn test_index_expressions() {
        let input = "myArray[1 + 1]";
        let prog = setup(input, 1);
        let exp = unwrap_first_expression_from_prog(&prog);

        match exp {
            Expression::Index(i) => {
                test_identifier(&i.left, "myArray");
                test_integer_infix(&i.index, 1, Token::Plus, 1);
            }
            _ => panic!("expected an index expression but got {:?}", exp),
        }
    }

    #[test]
    fn hash_literals() {
        let input = r#"{"one": 1, "two": 2, "three": 3, 4: 4, true: true}"#;
        let prog = setup(input, 1);
        let exp = unwrap_first_expression_from_prog(&prog);

        match exp {
            Expression::HashMap(h) => {
                assert_eq!(h.map.len(), 5);

                for (k, v) in &h.map {
                    match (&k, &v) {
                        (Expression::StringLiteral(key), Expression::IntegerLiteral(int)) => {
                            match key.as_str() {
                                "one" => assert_eq!(1, *int),
                                "two" => assert_eq!(2, *int),
                                "three" => assert_eq!(3, *int),
                                _ => panic!("unexpected key {}", k),
                            }
                        }
                        (Expression::IntegerLiteral(key), Expression::IntegerLiteral(int)) => {
                            assert_eq!(*key, *int);
                            assert_eq!(*int, 4);
                        }
                        (Expression::Boolean(key), Expression::Boolean(val)) => {
                            assert_eq!(key, val)
                        }
                        _ => panic!(
                            "expected key to be a string and value to be an int but got {:?} and {:?}",
                            k, v
                        ),
                    }
                }
            }
            _ => panic!("expected a hash literal but got {:?}", exp),
        }
    }

    #[test]
    fn hash_literal_with_expressions() {
        let input = r#"{"one": 0 + 1, "two": 10 - 8, "three": 15 / 5}"#;
        let prog = setup(input, 1);
        let exp = unwrap_first_expression_from_prog(&prog);

        match exp {
            Expression::HashMap(h) => {
                assert_eq!(h.map.len(), 3);

                for (k, v) in &h.map {
                    match (&k, &v) {
                        (Expression::StringLiteral(key), Expression::Infix { .. }) => {
                            match key.as_str() {
                                "one" => test_integer_infix(v, 0, Token::Plus, 1),
                                "two" => test_integer_infix(v, 10, Token::Minus, 8),
                                "three" => test_integer_infix(v, 15, Token::Slash, 5),
                                _ => panic!("unexpected key {}", key),
                            }
                        }
                        _ => panic!(
                            "expected key to be a string and value to be an infix expression but got {:?} and {:?}",
                            k, v
                        ),
                    }
                }
            }
            _ => panic!("expected a hash literal but got {:?}", exp),
        }
    }

    #[test]
    fn empty_hash_literal() {
        let input = "{}";
        let prog = setup(input, 1);
        let exp = unwrap_first_expression_from_prog(&prog);

        match exp {
            Expression::HashMap(h) => {
                assert_eq!(h.map.len(), 0)
            }
            _ => panic!("expected a hash literal but got {:?}", exp),
        }
    }

    fn setup(input: &str, stmt_count: usize) -> Program {
        let _ = env_logger::builder()
            .filter(None, log::LevelFilter::Debug)
            .is_test(true)
            .try_init();

        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let prog = p.parse_program().unwrap();

        if stmt_count != 0 && prog.statements.len() != stmt_count {
            panic!(
                "expected 1 statement for '{}' but got {:?}",
                input, prog.statements
            )
        }

        prog
    }

    fn unwrap_first_expression_from_prog(prog: &Program) -> &Expression {
        let s = prog.statements.first().unwrap();
        match s {
            Statement::Expression { value } => value,
            x => panic!("expected expression but got: {}", x),
        }
    }

    fn test_identifier(exp: &Expression, value: &str) {
        match exp {
            Expression::Identifier(ident) => {
                assert_eq!(value, ident, "expected {} but got {}", value, ident)
            }
            _ => panic!("expected identifier expression but got {:?}", exp),
        }
    }

    fn test_if_condition(exp: &Expression, op: Token, l: &str, r: &str) {
        match exp {
            Expression::Infix {
                left,
                operator,
                right,
            } => {
                test_identifier(&left, l);
                test_identifier(&right, r);
                if *operator != op {
                    panic!("expected {} operator but got {}", operator, op)
                }
            }
            _ => panic!("expected infix expression but got {:?}", exp),
        }
    }

    fn test_integer_literal(exp: &Expression, value: i64) {
        match exp {
            Expression::IntegerLiteral(int) => {
                assert_eq!(value, *int, "expected {} but got {}", value, int)
            }
            _ => panic!("expected integer literal {} but got {:?}", value, exp),
        }
    }

    fn test_boolean_literal(exp: &Expression, value: bool) {
        match exp {
            Expression::Boolean(val) => {
                assert_eq!(value, *val, "expected {} but got {}", value, val)
            }
            _ => panic!("expected boolean literal {} but got {:?}", value, exp),
        }
    }
}

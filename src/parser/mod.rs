use crate::{
    ast::{Expression, Program, Statement},
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

pub struct Parser<'a> {
    l: Lexer<'a>,

    current_token: Token,
    peek_token: Token,
}

impl<'a> Parser<'a> {
    fn new(mut l: Lexer<'a>) -> Self {
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

        debug!(
            "Got next_token, current_token: {} peek_token: {} from next_token call",
            self.current_token, self.peek_token
        );
    }

    fn parse(&mut self) -> ParserResult<Program> {
        let mut program = Program::default();
        let mut parse_error: Option<anyhow::Error> = None;

        while self.current_token != Token::Eof {
            match self.parse_statement() {
                Ok(statement) => program.statements.push(statement),
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

        if self.expect_peek_token_is(&Token::Semicolon) {
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

        while !self.expect_peek_token_is(&Token::Semicolon) && precedence < self.peek_precendence()
        {
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
            _ => None,
        }
    }

    fn current_prefix_fn(&mut self) -> Option<PrefixParseFn> {
        match self.current_token {
            Token::Ident(_) => Some(parse_identifier),
            Token::Int(_) => Some(parse_integer_literal),
            Token::Minus | Token::Bang => Some(parse_prefix_expression),
            _ => None,
        }
    }

    fn parse_return_statement(&mut self) -> ParserResult<Statement> {
        while !self.expect_current_token_is(&Token::Semicolon) {
            self.next_token();
        }

        Ok(Statement::Return {
            value: Expression::Empty,
        })
    }

    fn parse_let_statement(&mut self) -> ParserResult<Statement> {
        let name = self.expect_ident()?;
        self.expect_peek(Token::Assign)?;

        while !self.expect_current_token_is(&Token::Semicolon) {
            self.next_token();
        }

        Ok(Statement::Let {
            name,
            value: Expression::Empty,
        })
    }

    fn expect_peek(&mut self, tok: Token) -> ParserResult<()> {
        if self.expect_peek_token_is(&tok) {
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

    fn expect_current_token_is(&self, tok: &Token) -> bool {
        return self.current_token == *tok;
    }

    fn expect_peek_token_is(&self, tok: &Token) -> bool {
        return self.peek_token == *tok;
    }

    fn peek_precendence(&self) -> Precedence {
        Precedence::from_token(&self.peek_token)
    }

    fn current_precendence(&self) -> Precedence {
        Precedence::from_token(&self.current_token)
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

    fn setup(input: &str, stmt_count: usize) -> Program {
        let _ = env_logger::builder()
            .filter(None, log::LevelFilter::Debug)
            .is_test(true)
            .try_init();

        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let prog = p.parse().unwrap();

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

    fn test_integer_literal(exp: &Expression, value: i64) {
        match exp {
            Expression::IntegerLiteral(int) => {
                assert_eq!(value, *int, "expected {} but got {}", value, int)
            }
            _ => panic!("expected integer literal {} but got {:?}", value, exp),
        }
    }

    #[test]
    fn let_statement() {
        let prog = setup(
            "\
let x = 5;
let y = 10;
let foobar = 838383;",
            3,
        );

        let tests = vec!["x", "y", "foobar"];

        let mut itr = prog.statements.iter();

        for t in tests {
            match itr.next().unwrap() {
                Statement::Let { name, .. } => {
                    assert_eq!(name, t);
                }
                _ => panic!("unknown node"),
            }
        }
    }

    #[test]
    fn return_statement() {
        let prog = setup(
            "\
    return 5;
    return 10;
    return add(3,5);",
            3,
        );

        let mut itr = prog.statements.iter();

        let mut c = 0;
        while let Some(st) = itr.next() {
            match st {
                Statement::Return { .. } => {
                    c += 1;
                }
                _ => panic!("unknown node"),
            }
        }
        assert_eq!(c, 3)
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
        };
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
    fn infix_expressions() {
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
                        "expected {:?} operator but got {:?}",
                        t.operator, operator
                    );
                    test_integer_literal(left, t.left_value);
                    test_integer_literal(right, t.right_value);
                }
                exp => panic!("expected prefix expression but got {:?}", exp),
            }
        }
    }
}

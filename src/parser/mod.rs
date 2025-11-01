use anyhow::anyhow;
use log::debug;

use crate::{
    ast::{Expression, Program, Statement},
    lexer::Lexer,
    token::Token,
};

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
    Index,
}

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
            "Got next_token, current_token: {:?} peek_token: {:?} from next_token call",
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

    fn parse_expression(&mut self, _: Precedence) -> ParserResult<Expression> {
        let left_exp: Expression;
        if let Some(f) = self.current_prefix_fn() {
            left_exp = f(self)?;
        } else {
            return Err(anyhow!(
                "could not find prefix function for token: {:?}",
                self.current_token
            ));
        }

        Ok(left_exp)
    }

    fn current_prefix_fn(&mut self) -> Option<PrefixParseFn> {
        match self.current_token {
            Token::Ident(_) => Some(parse_identifier),
            Token::Int(_) => Some(parse_integer_literal),
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
                "expected token {:?} but received: {:?}",
                tok,
                self.peek_token
            ))
        }
    }

    fn expect_ident(&mut self) -> ParserResult<String> {
        let name = match &self.peek_token {
            Token::Ident(name) => name.to_string(),
            tok => return Err(anyhow!("expected ident token but received: {:?}", tok)),
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
}

fn parse_identifier(parser: &mut Parser<'_>) -> ParserResult<Expression> {
    if let Token::Ident(ref s) = parser.current_token {
        Ok(Expression::Identifier(s.clone()))
    } else {
        Err(anyhow!(
            "expected ident token but got: {:?}",
            parser.current_token
        ))
    }
}

fn parse_integer_literal(parser: &mut Parser<'_>) -> ParserResult<Expression> {
    if let Token::Int(i) = parser.current_token {
        Ok(Expression::IntegerLiteral(i))
    } else {
        Err(anyhow!(
            "expected ident token but got: {:?}",
            parser.current_token
        ))
    }
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
    fn test_integer_literal() {
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
}

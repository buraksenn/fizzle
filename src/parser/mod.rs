use anyhow::{Context, anyhow};
use log::debug;

use crate::{
    ast::{Expression, Program, Statement},
    lexer::Lexer,
    token::Token,
};

type ParserResult<T> = anyhow::Result<T>;

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
            _ => Err(anyhow!(
                "Got not supported token in parse_statement: {:?}",
                self.current_token
            )),
        }
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ast::Statement, lexer::Lexer};

    #[test]
    fn let_statement() {
        env_logger::builder()
            .filter(None, log::LevelFilter::Debug)
            .is_test(true)
            .try_init()
            .unwrap();

        let input = "\
let x = 5;
let y = 10;
let foobar = 838383;";

        let lexer = Lexer::new(input);
        let prog = Parser::new(lexer).parse().unwrap();

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
}

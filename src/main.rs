use fizzle::token::Token;
use std::{iter::Peekable, str::Chars};

struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
        }
    }

    fn eat_whitespace(&mut self) {
        while let Some(&ch) = self.chars.peek() {
            if ch.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.eat_whitespace();
        match self.chars.next() {
            Some('=') => Token::Assign,
            Some(';') => Token::Semicolon,
            Some(',') => Token::Comma,

            Some('(') => Token::Lparen,
            Some(')') => Token::Rparen,
            Some('{') => Token::Lbrace,
            Some('}') => Token::Rbrace,

            Some('+') => Token::Plus,
            Some('-') => Token::Minus,
            Some('!') => Token::Bang,
            Some('*') => Token::Asterisk,
            Some('/') => Token::Slash,
            Some('<') => Token::Lt,
            Some('>') => Token::Gt,

            Some(ch) => {
                if ch.is_alphabetic() {
                    let mut identifier = vec![ch];
                    while let Some(next_ch) = self.chars.next_if(|c| c.is_alphabetic()) {
                        identifier.push(next_ch);
                    }
                    return Token::lookup_ident(identifier.iter().collect());
                } else if ch.is_digit(10) {
                    let mut s = String::new();
                    s.push(ch);
                    while let Some(next_ch) = self.chars.next_if(|c| c.is_numeric()) {
                        s.push(next_ch)
                    }

                    return Token::Int(s.parse().unwrap());
                } else {
                    return Token::Illegal;
                }
            }
            _ => Token::Eof,
        }
    }
}

pub fn main() {}

#[cfg(test)]
mod tests {
    use super::{Lexer, Token};

    #[test]
    fn test_next_token_simple_symbols() {
        let input = "let five = 5;
        let ten = 10;
        let add = fn(x, y) {
        x + y;
        };
        let result = add(five, ten);
        !-/*5;
        5 < 10 > 5;

        if (5 < 10) {
        return true;
        } else {
        return false;
        }
        ";

        let tests = vec![
            Token::Let,
            Token::Ident("five".into()),
            Token::Assign,
            Token::Int(5),
            Token::Semicolon,
            Token::Let,
            Token::Ident("ten".into()),
            Token::Assign,
            Token::Int(10),
            Token::Semicolon,
            Token::Let,
            Token::Ident("add".into()),
            Token::Assign,
            Token::Function,
            Token::Lparen,
            Token::Ident("x".into()),
            Token::Comma,
            Token::Ident("y".into()),
            Token::Rparen,
            Token::Lbrace,
            Token::Ident("x".into()),
            Token::Plus,
            Token::Ident("y".into()),
            Token::Semicolon,
            Token::Rbrace,
            Token::Semicolon,
            Token::Let,
            Token::Ident("result".into()),
            Token::Assign,
            Token::Ident("add".into()),
            Token::Lparen,
            Token::Ident("five".into()),
            Token::Comma,
            Token::Ident("ten".into()),
            Token::Rparen,
            Token::Semicolon,
            Token::Bang,
            Token::Minus,
            Token::Slash,
            Token::Asterisk,
            Token::Int(5),
            Token::Semicolon,
            Token::Int(5),
            Token::Lt,
            Token::Int(10),
            Token::Gt,
            Token::Int(5),
            Token::Semicolon,
            Token::If,
            Token::Lparen,
            Token::Int(5),
            Token::Lt,
            Token::Int(10),
            Token::Rparen,
            Token::Lbrace,
            Token::Return,
            Token::True,
            Token::Semicolon,
            Token::Rbrace,
            Token::Else,
            Token::Lbrace,
            Token::Return,
            Token::False,
            Token::Semicolon,
            Token::Rbrace,
            Token::Eof,
        ];

        let mut lexer = Lexer::new(input);

        for expected_token in tests.into_iter() {
            let tok = lexer.next_token();

            assert_eq!(tok, expected_token);
        }
    }
}

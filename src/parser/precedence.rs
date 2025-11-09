use crate::token::Token;

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub enum Precedence {
    Lowest,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
    // Index,
}

impl Precedence {
    pub fn from_token(tok: &Token) -> Precedence {
        match tok {
            Token::Eq => Self::Equals,
            Token::Neq => Self::Equals,
            Token::Lt => Self::LessGreater,
            Token::Gt => Self::LessGreater,
            Token::Plus => Self::Sum,
            Token::Minus => Self::Sum,
            Token::Slash => Self::Product,
            Token::Asterisk => Self::Product,
            Token::Lparen => Self::Call,
            _ => Self::Lowest,
        }
    }
}

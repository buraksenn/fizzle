use std::fmt;

use crate::token::Token;

#[derive(Debug)]
pub enum Expression {
    Identifier(String),
    IntegerLiteral(i64),
    Prefix {
        operator: Token,
        operand: Box<Expression>,
    },
    Infix {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Empty,
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Expression::Identifier(s) => format!("{}", s),
            Expression::IntegerLiteral(i) => format!("{}", i),
            Expression::Prefix { operator, operand } => {
                format!("({}{})", operator, operand.as_ref())
            }
            Expression::Infix {
                left,
                operator,
                right,
            } => format!("({} {} {})", left.as_ref(), operator, right.as_ref(),),
            Expression::Empty => format!("nothing yet"),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub enum Statement {
    Let { name: String, value: Expression },
    Return { value: Expression },
    Expression { value: Expression },
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Statement::Let { name, value } => format!("let {} = {};", name, value),
            Statement::Return { value } => format!("return {};", value),
            Statement::Expression { value } => format!("{}", value),
        };
        write!(f, "{}", s)
    }
}

#[derive(Default)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let statements: Vec<String> = (&self.statements)
            .into_iter()
            .map(|stmt| stmt.to_string())
            .collect();
        write!(f, "{}", statements.join(""))
    }
}

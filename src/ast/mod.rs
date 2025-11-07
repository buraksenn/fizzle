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
    If(Box<IfExpression>),
    Function(Box<FunctionExpression>),
    Call(Box<CallExpression>),
    Boolean(bool),
    Empty,
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Expression::Identifier(s) => format!("{}", s),
            Expression::IntegerLiteral(i) => format!("{}", i),
            Expression::Boolean(val) => format!("{}", val),
            Expression::Prefix { operator, operand } => {
                format!("({}{})", operator, operand.as_ref())
            }
            Expression::Infix {
                left,
                operator,
                right,
            } => format!("({} {} {})", left.as_ref(), operator, right.as_ref(),),
            Expression::If(exp) => exp.to_string(),
            Expression::Function(f) => f.to_string(),
            Expression::Call(c) => c.to_string(),
            Expression::Empty => format!("nothing yet"),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub struct FunctionExpression {
    pub parameters: Vec<String>,
    pub body: BlockStatement,
}

impl fmt::Display for FunctionExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parameters: Vec<String> = (&self.parameters)
            .into_iter()
            .map(|pm| pm.to_string())
            .collect();
        write!(f, "fn({}) {{ {} }}", parameters.join(", "), self.body)
    }
}

#[derive(Debug)]
pub struct CallExpression {
    pub function: Expression,
    pub arguments: Vec<Expression>,
}

impl fmt::Display for CallExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args: Vec<String> = (&self.arguments)
            .into_iter()
            .map(|pm| pm.to_string())
            .collect();
        write!(f, "{}({})", self.function, args.join(", "))
    }
}

#[derive(Debug)]
pub struct IfExpression {
    pub condition: Expression,
    pub consequence: BlockStatement,
    pub alternative: Option<BlockStatement>,
}

impl fmt::Display for IfExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "if {} {}", self.condition, self.consequence)?;

        if let Some(ref stmt) = self.alternative {
            write!(f, "else {}", stmt)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
}

impl fmt::Display for BlockStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let statements: Vec<String> = (&self.statements)
            .into_iter()
            .map(|stmt| stmt.to_string())
            .collect();
        write!(f, "{}", statements.join(""))
    }
}

#[derive(Debug)]
pub enum Statement {
    Let { name: String, value: Expression },
    Return { value: Expression },
    Expression { value: Expression },
    Block(BlockStatement),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Statement::Let { name, value } => format!("let {} = {};", name, value),
            Statement::Return { value } => format!("return {};", value),
            Statement::Expression { value } => format!("{}", value),
            Statement::Block(b) => format!("{}", b.to_string()),
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

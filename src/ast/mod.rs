use std::fmt;

pub enum Expression {
    Empty,
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Expression::Empty => format!("nothing yet"),
        };
        write!(f, "{}", s)
    }
}

pub enum Statement {
    Let { name: String, value: Expression },
    Return { value: Expression },
    Expression { value: Expression },
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Statement::Let { name, value } => format!("name: {} value: {}", name, value),
            Statement::Return { value } => format!("return value: {}", value),
            Statement::Expression { value } => format!("expression: {}", value),
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

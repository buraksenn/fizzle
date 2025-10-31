pub enum Expression {
    Empty,
}

pub enum Statement {
    Let { name: String, value: Expression },
    Return { value: Expression },
}

#[derive(Default)]
pub struct Program {
    pub statements: Vec<Statement>,
}

use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
    ast::BlockStatement,
    object::{builtin::BuiltinFuncType, environment::Environment},
};

#[derive(Debug, Clone)]
pub enum Object {
    Integer(i64),
    String(String),
    Boolean(bool),
    Return(Rc<Object>),
    Function(Box<Function>),
    Builtin(BuiltinFuncType),
    Array(Rc<Vec<Rc<Object>>>),
    Null,
}

impl Object {
    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.to_string(),
            Object::Boolean(b) => b.to_string(),
            Object::String(s) => s.clone(),
            Object::Return(obj) => obj.to_string(),
            Object::Function(f) => f.inspect(),
            Object::Builtin(_) => "builtin function".into(),
            Object::Array(a) => a
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            Object::Null => "null".into(),
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inspect())
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub parameters: Vec<String>,
    pub body: BlockStatement,
    pub env: Rc<RefCell<Environment>>,
}

impl Function {
    fn inspect(&self) -> String {
        let params: Vec<String> = (&self.parameters)
            .into_iter()
            .map(|p| p.to_string())
            .collect();
        format!(
            "fn({}) {{\n{}\n}}",
            params.join(", "),
            self.body.to_string()
        )
    }
}

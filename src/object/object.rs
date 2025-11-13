use std::{
    cell::RefCell,
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
    rc::Rc,
};

use crate::{
    ast::BlockStatement,
    object::{builtin::Builtin, environment::Environment},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Object {
    Integer(i64),
    String(String),
    Boolean(bool),
    Return(Rc<Object>),
    Function(Box<Function>),
    Builtin(Builtin),
    Array(Rc<Vec<Rc<Object>>>),
    HashMap(Rc<HashMapObject>),
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
            Object::HashMap(hm) => hm
                .map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
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

impl PartialEq for Function {
    fn eq(&self, _other: &Function) -> bool {
        panic!("partial eq not implemented for function");
    }
}
impl Eq for Function {}

impl Hash for Function {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        panic!("hash for function not supported");
    }
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

#[derive(Debug, Clone)]
pub struct HashMapObject {
    pub map: HashMap<Rc<Object>, Rc<Object>>,
}

impl PartialEq for HashMapObject {
    fn eq(&self, _other: &HashMapObject) -> bool {
        panic!("partial eq not implemented for hash");
    }
}
impl Eq for HashMapObject {}

impl Hash for HashMapObject {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        panic!("hash for hash maps not supported");
    }
}

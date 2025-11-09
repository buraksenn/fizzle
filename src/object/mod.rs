use std::fmt;

#[derive(Debug)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    Return(Box<Object>),
    Null,
}

impl Object {
    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.to_string(),
            Object::Boolean(b) => b.to_string(),
            Object::Return(obj) => obj.to_string(),
            Object::Null => "null".into(),
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inspect())
    }
}

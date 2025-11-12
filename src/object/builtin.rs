use std::rc::Rc;

use anyhow::anyhow;

use crate::object::object::Object;

pub type BuiltinResult = Result<Rc<Object>, anyhow::Error>;
pub type BuiltinFuncType = fn(args: Vec<Rc<Object>>) -> BuiltinResult;

pub fn from_str(s: &str) -> Option<BuiltinFuncType> {
    match s {
        "len" => Some(len),
        _ => None,
    }
}

fn len(args: Vec<Rc<Object>>) -> BuiltinResult {
    if args.len() != 1 {
        return Err(anyhow!(
            "wrong number of arguments, got: {}, want = 1",
            args.len()
        ));
    }

    match &*args[0] {
        Object::String(s) => Ok(Rc::new(Object::Integer(s.len() as i64))),
        obj => Err(anyhow!("expected string but got object: {}", obj)),
    }
}

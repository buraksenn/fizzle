use std::rc::Rc;

use anyhow::anyhow;

use crate::object::object::Object;

pub type BuiltinResult = Result<Rc<Object>, anyhow::Error>;
pub type BuiltinFuncType = fn(args: Vec<Rc<Object>>) -> BuiltinResult;

#[derive(Hash, Eq, PartialEq, Clone, Debug, Copy)]
pub enum Builtin {
    Len,
    First,
    Last,
    Rest,
    Push,
    Puts,
}

impl Builtin {
    pub fn lookup(name: &str) -> Option<Object> {
        match name {
            "len" => Some(Object::Builtin(Builtin::Len)),
            "first" => Some(Object::Builtin(Builtin::First)),
            "last" => Some(Object::Builtin(Builtin::Last)),
            "rest" => Some(Object::Builtin(Builtin::Rest)),
            "push" => Some(Object::Builtin(Builtin::Push)),
            "puts" => Some(Object::Builtin(Builtin::Puts)),
            _ => None,
        }
    }

    pub fn to_func(&self) -> BuiltinFuncType {
        match self {
            Builtin::Len => len,
            Builtin::First => first,
            Builtin::Last => last,
            Builtin::Rest => rest,
            Builtin::Push => push,
            Builtin::Puts => puts,
        }
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
        Object::Array(a) => Ok(Rc::new(Object::Integer(a.len() as i64))),
        obj => Err(anyhow!("expected string or array but got object: {}", obj)),
    }
}

fn first(args: Vec<Rc<Object>>) -> BuiltinResult {
    if args.len() != 1 {
        return Err(anyhow!(
            "wrong number of arguments, got: {}, want = 1",
            args.len()
        ));
    }
    match &*args[0] {
        Object::Array(a) => match a.first() {
            Some(el) => Ok(Rc::clone(el)),
            None => Ok(Rc::new(Object::Null)),
        },
        obj => Err(anyhow!("expected array but got object: {}", obj)),
    }
}

fn last(args: Vec<Rc<Object>>) -> BuiltinResult {
    if args.len() != 1 {
        return Err(anyhow!(
            "wrong number of arguments, got: {}, want = 1",
            args.len()
        ));
    }
    match &*args[0] {
        Object::Array(a) => match a.last() {
            Some(el) => Ok(Rc::clone(el)),
            None => Ok(Rc::new(Object::Null)),
        },
        obj => Err(anyhow!("expected array but got object: {}", obj)),
    }
}

fn rest(args: Vec<Rc<Object>>) -> BuiltinResult {
    if args.len() != 1 {
        return Err(anyhow!(
            "wrong number of arguments, got: {}, want = 1",
            args.len()
        ));
    }
    match &*args[0] {
        Object::Array(a) => {
            if a.len() <= 1 {
                return Ok(Rc::new(Object::Array(Rc::new(vec![]))));
            }

            Ok(Rc::new(Object::Array(Rc::new(a[1..].to_vec().clone()))))
        }
        obj => Err(anyhow!("expected array but got object: {}", obj)),
    }
}

fn push(args: Vec<Rc<Object>>) -> BuiltinResult {
    if args.len() != 2 {
        return Err(anyhow!(
            "wrong number of arguments, got: {}, want = 2",
            args.len()
        ));
    }
    match &*args[0] {
        Object::Array(a) => {
            let mut copied = a.to_vec();
            copied.push(args[1].clone());

            Ok(Rc::new(Object::Array(Rc::new(copied))))
        }
        obj => Err(anyhow!("expected array but got object: {}", obj)),
    }
}

fn puts(args: Vec<Rc<Object>>) -> BuiltinResult {
    if args.len() == 0 {
        return Err(anyhow!("expected at least one number but got zero"));
    }

    for obj in args.iter() {
        println!("{}", obj.inspect());
    }

    Ok(Rc::new(Object::Null))
}

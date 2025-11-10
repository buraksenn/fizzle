use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::object::object::Object;

#[derive(Debug)]
pub struct Environment {
    store: HashMap<String, Rc<Object>>,
    pub outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn new_enclosed(outer: Rc<RefCell<Environment>>) -> Self {
        Environment {
            store: HashMap::new(),
            outer: Some(Rc::clone(&outer)),
        }
    }

    pub fn get(&self, key: &str) -> Option<Rc<Object>> {
        match self.store.get(key) {
            Some(val) => Some(Rc::clone(val)),
            None => match &self.outer {
                Some(o) => o.borrow().get(key),
                None => None,
            },
        }
    }

    pub fn set(&mut self, key: String, val: Rc<Object>) {
        self.store.insert(key, val);
    }
}

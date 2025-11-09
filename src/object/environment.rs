use std::{collections::HashMap, rc::Rc};

use crate::object::object::Object;

#[derive(Debug)]
pub struct Environment {
    store: HashMap<String, Rc<Object>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<Rc<Object>> {
        self.store.get(key).cloned()
    }

    pub fn set(&mut self, key: String, val: Rc<Object>) {
        self.store.insert(key, val);
    }
}

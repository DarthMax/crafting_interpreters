use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::evaluation::Value;

pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    variables: HashMap<String, Option<Value>>,
}

impl Environment {
    pub(crate) fn empty() -> Environment {
        Environment {
            parent: None,
            variables: HashMap::new(),
        }
    }

    pub(crate) fn wrap(parent: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            parent: Some(parent),
            variables: HashMap::new(),
        }
    }

    pub fn register(&mut self, key: String, value: Option<Value>) {
        self.variables.insert(key, value);
    }

    pub fn assign(&mut self, key: &String, value: Value) -> bool {
        if self.variables.contains_key(key) {
            self.variables.insert(key.clone(), Some(value));
            true
        } else {
            match &self.parent {
                Some(p) => p.borrow_mut().assign(key, value),
                None => false,
            }
        }
    }

    pub fn get(&self, key: &String) -> Option<Option<Value>> {
        if self.variables.contains_key(key) {
            self.variables.get(key).cloned()
        } else {
            match &self.parent {
                Some(p) => p.borrow().get(key),
                None => None,
            }
        }
    }
}

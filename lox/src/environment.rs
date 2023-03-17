use std::collections::HashMap;

use crate::evaluation::ValueNode;

pub struct Environment {
    pub variables: HashMap<String, Option<ValueNode>>,
}

impl Environment {
    pub(crate) fn empty() -> Environment {
        Environment {
            variables: HashMap::new(),
        }
    }

    pub fn register(&mut self, key: String, value: Option<ValueNode>) {
        self.variables.insert(key, value);
    }

    pub fn get(&self, key: &String) -> Option<&Option<ValueNode>> {
        self.variables.get(key)
    }
}

use std::cell::RefCell;
use std::rc::Rc;

use crate::environment::Environment;
use crate::evaluation::{evaluate_statement, EvaluationResult};
use crate::statement::Statement;
use crate::value::{Value, ValueNode};

pub(crate) trait Callable {
    fn call(&self, arguments: Vec<ValueNode>) -> EvaluationResult<Value>;

    fn arity(&self) -> usize;
}

pub struct FunctionContainer {
    pub id: String,
    parameters: Vec<String>,
    body: Rc<Statement>,
    closure: Rc<RefCell<Environment>>,
}

impl FunctionContainer {
    pub(crate) fn new(
        name: &str,
        parameters: &[String],
        body: Rc<Statement>,
        closure: Rc<RefCell<Environment>>,
    ) -> FunctionContainer {
        FunctionContainer {
            id: name.to_string(),
            parameters: parameters.to_owned(),
            body,
            closure,
        }
    }
}

impl PartialEq for FunctionContainer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Callable for FunctionContainer {
    fn call(&self, arguments: Vec<ValueNode>) -> EvaluationResult<Value> {
        let mut env = Environment::wrap(self.closure.clone());

        for (key, value) in self.parameters.iter().zip(arguments) {
            env.register(key.to_string(), Some(value.value))
        }

        evaluate_statement(&self.body, Rc::new(RefCell::new(env)))
    }

    fn arity(&self) -> usize {
        self.parameters.len()
    }
}

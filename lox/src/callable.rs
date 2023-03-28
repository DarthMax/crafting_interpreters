use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

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
    body: Arc<Statement>,
}

impl FunctionContainer {
    pub(crate) fn new(
        name: &str,
        parameters: &[String],
        body: Arc<Statement>,
    ) -> FunctionContainer {
        FunctionContainer {
            id: name.to_string(),
            parameters: parameters.to_owned(),
            body,
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
        let mut env = Environment::empty();

        for (key, value) in self.parameters.iter().zip(arguments) {
            env.register(key.to_string(), Some(value.value))
        }

        evaluate_statement(&self.body, Rc::new(RefCell::new(env)))
    }

    fn arity(&self) -> usize {
        self.parameters.len()
    }
}

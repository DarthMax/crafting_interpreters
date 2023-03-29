use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use crate::callable::{Callable, FunctionContainer};
use crate::error::RuntimeError;
use crate::evaluation::ReturnOrError::{Error, Return};
use crate::evaluation::{EvaluationResult, ReturnOrError};
use crate::expression::LiteralType;
use crate::position::Position;

#[derive(PartialEq, Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    Str(Rc<str>),
    Function(Rc<FunctionContainer>),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => f.write_str("Nil"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Str(str) => write!(f, "{str}"),
            Value::Function(fun) => write!(f, "fun {}", fun.id),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => f.write_str("Nil"),
            Value::Boolean(b) => write!(f, "{b}:Boolean"),
            Value::Number(n) => write!(f, "{n}:Number"),
            Value::Str(str) => write!(f, "{str}:String"),
            Value::Function(fun) => write!(f, "fun {}", fun.id),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) struct ValueNode {
    pub(crate) value: Value,
    pub(crate) position: Position,
}

impl Display for ValueNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl ValueNode {
    pub(crate) fn new(value: Value, position: &Position) -> ValueNode {
        ValueNode {
            value,
            position: position.clone(),
        }
    }

    pub(crate) fn from_literal(literal: &LiteralType, position: &Position) -> Self {
        let value = match literal {
            LiteralType::NumberLit(value) => Value::Number(*value),
            LiteralType::StringLit(value) => Value::Str(value.as_str().into()),
            LiteralType::TrueLit => Value::Boolean(true),
            LiteralType::FalseLit => Value::Boolean(false),
            LiteralType::NilLit => Value::Nil,
        };

        ValueNode::new(value, position)
    }

    pub(crate) fn as_number(&self) -> EvaluationResult<f64> {
        match self.value {
            Value::Number(num) => Ok(num),
            _ => Err(Error(RuntimeError::type_error(self, "Number".to_string()))),
        }
    }

    pub(crate) fn as_boolean(&self) -> EvaluationResult<bool> {
        match self.value {
            Value::Boolean(b) => Ok(b),
            Value::Nil => Ok(false),
            _ => Err(Error(RuntimeError::type_error(self, "Boolean".to_string()))),
        }
    }

    pub(crate) fn as_str(&self) -> EvaluationResult<Rc<str>> {
        match &self.value {
            Value::Str(str) => Ok(str.clone()),
            _ => Err(Error(RuntimeError::type_error(self, "String".to_string()))),
        }
    }

    pub(crate) fn call(&self, arguments: Vec<ValueNode>) -> EvaluationResult<Value> {
        match &self.value {
            Value::Function(container) => match container.call(arguments) {
                Ok(v) => Ok(v),
                Err(Return(r)) => Ok(r),
                error => error,
            },
            _ => Err(Error(RuntimeError::type_error(
                self,
                "Callable".to_string(),
            ))),
        }
    }

    pub(crate) fn negative(&self) -> EvaluationResult<Value> {
        Ok(Value::Number(-self.as_number()?))
    }

    pub(crate) fn add(&self, other: &ValueNode) -> EvaluationResult<Value> {
        match &self.value {
            Value::Number(l) => {
                let added = l + other.as_number()?;
                Ok(Value::Number(added))
            }
            Value::Str(l) => {
                let appended = format!("{}{}", l, other.as_str()?).into();
                Ok(Value::Str(appended))
            }
            _ => Err(Error(RuntimeError::type_error(self, "Number".to_string()))),
        }
    }

    pub(crate) fn subtract(&self, other: &ValueNode) -> EvaluationResult<Value> {
        Ok(Value::Number(self.as_number()? - other.as_number()?))
    }

    pub(crate) fn multiply(&self, other: &ValueNode) -> EvaluationResult<Value> {
        match &self.value {
            Value::Number(l) => Ok(Value::Number(l * other.as_number()?)),
            Value::Str(l) => Ok(Value::Str(l.repeat(other.as_number()? as usize).into())),
            _ => Err(Error(RuntimeError::type_error(
                self,
                "Number or String".to_string(),
            ))),
        }
    }

    pub(crate) fn divide(&self, other: &ValueNode) -> EvaluationResult<Value> {
        Ok(Value::Number(self.as_number()? / other.as_number()?))
    }

    pub(crate) fn equals(&self, other: &ValueNode) -> EvaluationResult<Value> {
        Ok(Value::Boolean(self.eq(other)))
    }

    pub(crate) fn not_equals(&self, other: &ValueNode) -> EvaluationResult<Value> {
        Ok(Value::Boolean(!self.eq(other)))
    }

    pub(crate) fn less_than(&self, other: &ValueNode) -> EvaluationResult<Value> {
        let b = self
            .compare(other)
            .map_or(false, |ordering| ordering.is_lt());

        Ok(Value::Boolean(b))
    }

    pub(crate) fn less_than_or_equals(&self, other: &ValueNode) -> EvaluationResult<Value> {
        let b = self
            .compare(other)
            .map_or(false, |ordering| ordering.is_lt() || ordering.is_eq());

        Ok(Value::Boolean(b))
    }

    pub(crate) fn greater_than(&self, other: &ValueNode) -> EvaluationResult<Value> {
        let b = self
            .compare(other)
            .map_or(false, |ordering| ordering.is_gt());

        Ok(Value::Boolean(b))
    }

    pub(crate) fn greater_than_or_equals(&self, other: &ValueNode) -> EvaluationResult<Value> {
        let b = self
            .compare(other)
            .map_or(false, |ordering| ordering.is_gt() || ordering.is_eq());

        Ok(Value::Boolean(b))
    }

    pub(crate) fn compare(&self, other: &ValueNode) -> Option<Ordering> {
        match (&self.value, &other.value) {
            // (Value::Nil, Value::Nil) => Some(Ordering::Equal),
            (Value::Number(l), Value::Number(r)) => l.partial_cmp(r),
            (Value::Boolean(l), Value::Boolean(r)) => l.partial_cmp(r),
            (Value::Str(l), Value::Str(r)) => l.partial_cmp(r),
            _ => None,
        }
    }

    pub(crate) fn not(&self) -> EvaluationResult<Value> {
        Ok(Value::Boolean(!self.as_boolean()?))
    }
}

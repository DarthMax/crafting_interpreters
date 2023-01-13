use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use crate::error::{LoxError, RuntimeError};
use crate::evaluation::Value::{Boolean, Nil, Number, Str};
use crate::expression::LiteralType::*;
use crate::expression::{BinaryOp, Expression, ExpressionNode, LiteralType, UnaryOp};
use crate::position::Position;

pub type EvaluationResult<T> = Result<T, LoxError>;

#[derive(PartialEq, Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    Str(Rc<str>),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Nil => f.write_str("Nil"),
            Boolean(b) => write!(f, "{}", b),
            Number(n) => write!(f, "{}", n),
            Str(str) => write!(f, "{}", str),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Nil => f.write_str("Nil"),
            Boolean(b) => write!(f, "{}:Boolean", b),
            Number(n) => write!(f, "{}:Number", n),
            Str(str) => write!(f, "{}:String", str),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct ValueNode {
    pub(crate) value: Value,
    pub(crate) position: Position,
}

impl Display for ValueNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl ValueNode {
    fn new(value: Value, position: &Position) -> ValueNode {
        ValueNode {
            value,
            position: position.clone(),
        }
    }

    fn from_literal(literal: &LiteralType, position: &Position) -> Self {
        let value = match literal {
            NumberLit(value) => Number(*value),
            StringLit(value) => Str(value.as_str().into()),
            TrueLit => Boolean(true),
            FalseLit => Boolean(false),
            NilLit => Nil,
        };

        ValueNode::new(value, position)
    }

    fn as_number(&self) -> EvaluationResult<f64> {
        match self.value {
            Number(num) => Ok(num),
            _ => Err(RuntimeError::type_error(self, "Number".to_string())),
        }
    }

    fn as_boolean(&self) -> EvaluationResult<bool> {
        match self.value {
            Boolean(b) => Ok(b),
            Nil => Ok(false),
            _ => Err(RuntimeError::type_error(self, "Boolean".to_string())),
        }
    }

    fn as_str(&self) -> EvaluationResult<Rc<str>> {
        match &self.value {
            Str(str) => Ok(str.clone()),
            _ => Err(RuntimeError::type_error(self, "String".to_string())),
        }
    }

    fn negative(&self) -> EvaluationResult<Value> {
        Ok(Number(-self.as_number()?))
    }

    fn add(&self, other: &ValueNode) -> EvaluationResult<Value> {
        match &self.value {
            Number(l) => {
                let added = l + other.as_number()?;
                Ok(Number(added))
            }
            Str(l) => {
                let appended = format!("{}{}", l, other.as_str()?).into();
                Ok(Str(appended))
            }
            _ => Err(RuntimeError::type_error(self, "Number".to_string())),
        }
    }

    fn subtract(&self, other: &ValueNode) -> EvaluationResult<Value> {
        Ok(Number(self.as_number()? - other.as_number()?))
    }

    fn multiply(&self, other: &ValueNode) -> EvaluationResult<Value> {
        match &self.value {
            Number(l) => Ok(Number(l * other.as_number()?)),
            Str(l) => Ok(Str(l.repeat(other.as_number()? as usize).into())),
            _ => Err(RuntimeError::type_error(
                self,
                "Number or String".to_string(),
            )),
        }
    }

    fn divide(&self, other: &ValueNode) -> EvaluationResult<Value> {
        Ok(Number(self.as_number()? / other.as_number()?))
    }

    fn equals(&self, other: &ValueNode) -> EvaluationResult<Value> {
        Ok(Boolean(self.eq(other)))
    }

    fn not_equals(&self, other: &ValueNode) -> EvaluationResult<Value> {
        Ok(Boolean(!self.eq(other)))
    }

    fn less_than(&self, other: &ValueNode) -> EvaluationResult<Value> {
        let b = self
            .compare(other)
            .map_or(false, |ordering| ordering.is_lt());

        Ok(Boolean(b))
    }

    fn less_than_or_equals(&self, other: &ValueNode) -> EvaluationResult<Value> {
        let b = self
            .compare(other)
            .map_or(false, |ordering| ordering.is_lt() || ordering.is_eq());

        Ok(Boolean(b))
    }

    fn greater_than(&self, other: &ValueNode) -> EvaluationResult<Value> {
        let b = self
            .compare(other)
            .map_or(false, |ordering| ordering.is_gt());

        Ok(Boolean(b))
    }

    fn greater_than_or_equals(&self, other: &ValueNode) -> EvaluationResult<Value> {
        let b = self
            .compare(other)
            .map_or(false, |ordering| ordering.is_gt() || ordering.is_eq());

        Ok(Boolean(b))
    }

    fn compare(&self, other: &ValueNode) -> Option<Ordering> {
        match (&self.value, &other.value) {
            // (Value::Nil, Value::Nil) => Some(Ordering::Equal),
            (Number(l), Number(r)) => l.partial_cmp(r),
            (Boolean(l), Boolean(r)) => l.partial_cmp(r),
            (Str(l), Str(r)) => l.partial_cmp(r),
            _ => None,
        }
    }

    fn not(&self) -> EvaluationResult<Value> {
        Ok(Boolean(!self.as_boolean()?))
    }
}

pub fn evaluate(expr: &ExpressionNode) -> EvaluationResult<ValueNode> {
    match &expr.expression {
        Expression::Literal(lit) => {
            let value_node: ValueNode = ValueNode::from_literal(lit, &expr.position);
            Ok(value_node)
        }
        Expression::Grouping(inner) => evaluate(&inner),
        Expression::Unary { inner, op, .. } => {
            let inner_value = evaluate(inner)?;
            let value = match op {
                UnaryOp::Negative => inner_value.negative(),
                UnaryOp::Not => inner_value.not(),
            };
            Ok(ValueNode::new(value?, &expr.position))
        }
        Expression::Binary {
            left, right, op, ..
        } => {
            let left_value = evaluate(left)?;
            let right_value = evaluate(right)?;

            let value = match op {
                BinaryOp::Equals => left_value.equals(&right_value),
                BinaryOp::NotEquals => left_value.not_equals(&right_value),
                BinaryOp::LessThan => left_value.less_than(&right_value),
                BinaryOp::LessThanOrEquals => left_value.less_than_or_equals(&right_value),
                BinaryOp::GreaterThan => left_value.greater_than(&right_value),
                BinaryOp::GreaterThanOrEquals => left_value.greater_than_or_equals(&right_value),
                BinaryOp::Add => left_value.add(&right_value),
                BinaryOp::Subtract => left_value.subtract(&right_value),
                BinaryOp::Multiply => left_value.multiply(&right_value),
                BinaryOp::Divide => left_value.divide(&right_value),
            };
            Ok(ValueNode::new(value?, &expr.position))
        }
    }
}
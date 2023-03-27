use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use crate::environment::Environment;
use crate::error::LoxError;
use crate::error::RuntimeError;
use crate::evaluation::Value::{Boolean, Function, Nil, Number, Str};
use crate::expression::LiteralType::*;
use crate::expression::{BinaryOp, Expression, ExpressionNode, LiteralType, LogicalOp, UnaryOp};
use crate::position::Position;
use crate::statement::Statement;

pub type EvaluationResult<T> = Result<T, LoxError>;

trait Callable {
    fn call(&self, arguments: Vec<ValueNode>) -> EvaluationResult<Value>;

    fn arity(&self) -> usize;
}

pub struct FunctionContainer {
    pub(crate) id: String,
    parameters: Vec<String>,
    body: Rc<Statement>,
}

impl FunctionContainer {
    fn new(name: &str, parameters: &Vec<String>, body: Rc<Statement>) -> FunctionContainer {
        FunctionContainer {
            id: name.to_string(),
            parameters: parameters.clone(),
            body: body,
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
            Nil => f.write_str("Nil"),
            Boolean(b) => write!(f, "{b}"),
            Number(n) => write!(f, "{n}"),
            Str(str) => write!(f, "{str}"),
            Function(fun) => write!(f, "fun {}", fun.id),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Nil => f.write_str("Nil"),
            Boolean(b) => write!(f, "{b}:Boolean"),
            Number(n) => write!(f, "{n}:Number"),
            Str(str) => write!(f, "{str}:String"),
            Function(fun) => write!(f, "fun {}", fun.id),
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

    fn call(&self, arguments: Vec<ValueNode>) -> EvaluationResult<Value> {
        match &self.value {
            Function(container) => container.call(arguments),
            _ => Err(RuntimeError::type_error(self, "Callable".to_string())),
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

pub(crate) fn evaluate(
    statements: &Vec<Statement>,
    env: Rc<RefCell<Environment>>,
) -> EvaluationResult<Value> {
    let mut result: EvaluationResult<Value> = Ok(Nil);

    for stmt in statements {
        result = Ok(evaluate_statement(stmt, env.clone())?)
    }

    result
}

fn evaluate_statement(stmt: &Statement, env: Rc<RefCell<Environment>>) -> EvaluationResult<Value> {
    match stmt {
        Statement::Print(expr) => {
            let inner_value = evaluate_expression(expr, env)?;
            println!("{inner_value}");
            Ok(inner_value.value)
        }
        Statement::Expression(expr) => Ok(evaluate_expression(expr, env)?.value),
        Statement::Var { name, initializer } => {
            let initializer = match initializer {
                Some(expr) => Some(evaluate_expression(expr, env.clone())?.value),
                _ => None,
            };

            env.borrow_mut().register(name.to_string(), initializer);

            Ok(Nil)
        }
        Statement::Block(statements) => {
            let block_env = Rc::new(RefCell::new(Environment::wrap(env)));

            for stmt in statements {
                evaluate_statement(stmt, block_env.clone())?;
            }

            Ok(Nil)
        }
        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let condition = evaluate_expression(condition, env.clone())?;

            if condition.as_boolean()? {
                evaluate_statement(then_branch, env.clone())
            } else {
                match else_branch {
                    Some(else_branch) => evaluate_statement(else_branch, env.clone()),
                    _ => Ok(Nil),
                }
            }
        }
        Statement::While { condition, body } => {
            while evaluate_expression(condition, env.clone())?.as_boolean()? {
                evaluate_statement(body, env.clone())?;
            }

            Ok(Nil)
        }
        Statement::Function {
            name,
            parameters,
            body,
        } => {
            let container = FunctionContainer::new(name, parameters, body.clone());
            env.borrow_mut()
                .register(name.to_string(), Some(Function(Rc::new(container))));

            Ok(Nil)
        }
    }
}

fn evaluate_expression(
    expr: &ExpressionNode,
    env: Rc<RefCell<Environment>>,
) -> EvaluationResult<ValueNode> {
    match &expr.expression {
        Expression::Literal(lit) => {
            let value_node: ValueNode = ValueNode::from_literal(lit, &expr.position);
            Ok(value_node)
        }
        Expression::Grouping(inner) => evaluate_expression(inner, env),
        Expression::Unary { inner, op, .. } => {
            let inner_value = evaluate_expression(inner, env)?;
            let value = match op {
                UnaryOp::Negative => inner_value.negative(),
                UnaryOp::Not => inner_value.not(),
            };
            Ok(ValueNode::new(value?, &expr.position))
        }
        Expression::Binary {
            left, right, op, ..
        } => {
            let left_value = evaluate_expression(left, env.clone())?;
            let right_value = evaluate_expression(right, env)?;

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
        Expression::Logical { left, right, op } => {
            let left_value = evaluate_expression(left, env.clone())?;

            match op {
                LogicalOp::And => {
                    if !left_value.as_boolean()? {
                        return Ok(ValueNode::new(Boolean(false), &expr.position));
                    }
                }
                LogicalOp::Or => {
                    if left_value.as_boolean()? {
                        return Ok(ValueNode::new(Boolean(true), &expr.position));
                    }
                }
            }

            let right_value = evaluate_expression(right, env)?;
            Ok(ValueNode::new(right_value.value, &expr.position))
        }
        Expression::Variable(name) => match env.borrow().get(name) {
            Some(Some(value)) => Ok(ValueNode::new(value, &expr.position)),
            Some(None) => Err(RuntimeError::uninitialized_variable(
                name.to_string(),
                expr.position.clone(),
            )),
            None => Err(RuntimeError::unknown_identifier(
                name.to_string(),
                expr.position.clone(),
            )),
        },
        Expression::Assignment { name, value } => {
            let value = evaluate_expression(value, env.clone())?;
            match env.borrow_mut().assign(name, value.value) {
                true => Ok(ValueNode::new(Nil, &expr.position)),
                false => Err(RuntimeError::unknown_identifier(
                    name.to_string(),
                    expr.position.clone(),
                )),
            }
        }
        Expression::Call { callee, arguments } => {
            let callee_expr = evaluate_expression(callee, env.clone())?;

            let argument_values = arguments
                .iter()
                .map(|arg| evaluate_expression(arg, env.clone()))
                .collect::<EvaluationResult<Vec<ValueNode>>>()?;

            let value = callee_expr.call(argument_values)?;

            Ok(ValueNode::new(value, &expr.position))
        }
    }
}

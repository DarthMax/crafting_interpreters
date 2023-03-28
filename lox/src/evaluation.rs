use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::callable::FunctionContainer;
use crate::environment::Environment;
use crate::error::LoxError;
use crate::error::ReturnUnwind;
use crate::error::RuntimeError;
use crate::evaluation::Value::{Boolean, Function, Nil};
use crate::expression::{BinaryOp, Expression, ExpressionNode, LogicalOp, UnaryOp};
use crate::statement::Statement;
use crate::value::{Value, ValueNode};

pub type EvaluationResult<T> = Result<T, LoxError>;

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

pub(crate) fn evaluate_statement(
    stmt: &Statement,
    env: Rc<RefCell<Environment>>,
) -> EvaluationResult<Value> {
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
                evaluate_statement(then_branch, env)
            } else {
                match else_branch {
                    Some(else_branch) => evaluate_statement(else_branch, env),
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
                .register(name.to_string(), Some(Function(Arc::new(container))));

            Ok(Nil)
        }
        Statement::Return(return_expression) => {
            let value = match return_expression {
                Some(e) => evaluate_expression(e, env)?.value,
                _ => Nil,
            };

            Err(ReturnUnwind::return_unwind(value))
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

use std::fmt;
use std::fmt::Formatter;

use crate::expression::BinaryOp::*;
use crate::expression::Expression::*;
use crate::expression::LiteralType::*;
use crate::expression::UnaryOp::*;
use crate::position::Position;
use crate::token::TokenType;
use crate::token::TokenType::*;

pub struct ExpressionNode {
    pub expression: Expression,
    pub position: Position,
}

impl ExpressionNode {
    pub fn new(expression: Expression, position: &Position) -> ExpressionNode {
        ExpressionNode {
            expression,
            position: position.clone(),
        }
    }
}

pub enum Expression {
    Unary {
        inner: Box<ExpressionNode>,
        op: UnaryOp,
    },
    Binary {
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
        op: BinaryOp,
    },
    Logical {
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
        op: LogicalOp,
    },
    Literal(LiteralType),
    Grouping(Box<ExpressionNode>),
    Variable(String),
    Assignment {
        name: String,
        value: Box<ExpressionNode>,
    },
}

impl ExpressionNode {
    pub fn pretty(&self) -> String {
        fn pretty(expr: &ExpressionNode, level: u32) -> String {
            let mut prefix = if level == 0 {
                "".to_string()
            } else {
                let mut prefix = "   ".repeat(level as usize);
                prefix.push_str("|_ ");
                prefix
            };

            let thing = match &expr.expression {
                Unary { inner, op, .. } => {
                    format!(
                        "Unary {} ({}:{})\n{}",
                        op,
                        expr.position.absolute,
                        expr.position.length,
                        pretty(inner, level + 1)
                    )
                }
                Binary {
                    left, right, op, ..
                } => {
                    format!(
                        "Binary {} ({}:{})\n{}\n{}",
                        op,
                        expr.position.absolute,
                        expr.position.length,
                        pretty(left, level + 1),
                        pretty(right, level + 1)
                    )
                }
                Logical {
                    left, right, op, ..
                } => {
                    format!(
                        "Logical {} ({}:{})\n{}\n{}",
                        op,
                        expr.position.absolute,
                        expr.position.length,
                        pretty(left, level + 1),
                        pretty(right, level + 1)
                    )
                }
                Literal(value) => format!(
                    "{} ({}:{})",
                    value, expr.position.absolute, expr.position.length,
                ),
                Grouping(inner) => {
                    format!(
                        "Group  ({}:{})\n{}",
                        expr.position.absolute,
                        expr.position.length,
                        pretty(inner, level + 1)
                    )
                }
                Variable(identifier) => {
                    format!(
                        "Variable: {} ({}:{})",
                        identifier, expr.position.absolute, expr.position.length
                    )
                }
                Assignment { name, value } => {
                    format!(
                        "Assignment: {} ({}:{})\n{}",
                        name,
                        expr.position.absolute,
                        expr.position.length,
                        pretty(value, level + 1),
                    )
                }
            };

            prefix.push_str(&thing);

            prefix
        }

        pretty(self, 0)
    }
}

pub enum LiteralType {
    NumberLit(f64),
    StringLit(String),
    TrueLit,
    FalseLit,
    NilLit,
}

impl fmt::Display for LiteralType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NumberLit(value) => write!(f, "{value}"),
            StringLit(value) => write!(f, "\"{value}\""),
            TrueLit => write!(f, "true"),
            FalseLit => write!(f, "false"),
            NilLit => write!(f, "nil"),
        }
    }
}

pub enum BinaryOp {
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEquals,
    GreaterThan,
    GreaterThanOrEquals,
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Equals => write!(f, "=="),
            NotEquals => write!(f, "!="),
            LessThan => write!(f, "<"),
            LessThanOrEquals => write!(f, "<="),
            GreaterThan => write!(f, ">"),
            GreaterThanOrEquals => write!(f, ">="),
            Add => write!(f, "+"),
            Subtract => write!(f, "-"),
            Multiply => write!(f, "*"),
            Divide => write!(f, "/"),
        }
    }
}

impl TryFrom<&TokenType> for BinaryOp {
    type Error = &'static str;

    fn try_from(token_type: &TokenType) -> Result<Self, Self::Error> {
        match token_type {
            EqualEqual => Ok(Equals),
            BangEqual => Ok(NotEquals),
            Greater => Ok(GreaterThan),
            GreaterEqual => Ok(GreaterThanOrEquals),
            Less => Ok(LessThan),
            LessEqual => Ok(LessThanOrEquals),
            Minus => Ok(Subtract),
            Plus => Ok(Add),
            Slash => Ok(Divide),
            Star => Ok(Multiply),
            _ => Err("Could not do this"),
        }
    }
}

pub enum LogicalOp {
    And,
    Or,
}

impl fmt::Display for LogicalOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LogicalOp::And => write!(f, "and"),
            LogicalOp::Or => write!(f, "or"),
        }
    }
}

impl TryFrom<&TokenType> for LogicalOp {
    type Error = &'static str;

    fn try_from(token_type: &TokenType) -> Result<Self, Self::Error> {
        match token_type {
            And => Ok(LogicalOp::And),
            Or => Ok(LogicalOp::Or),
            _ => Err("Could not do this"),
        }
    }
}

pub enum UnaryOp {
    Not,
    Negative,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Not => write!(f, "Not"),
            Negative => write!(f, "Negative"),
        }
    }
}

impl TryFrom<&TokenType> for UnaryOp {
    type Error = String;

    fn try_from(value: &TokenType) -> Result<Self, Self::Error> {
        match value {
            Bang => Ok(Not),
            Minus => Ok(Negative),
            other => Err(format!("Cannot convert {other:?} into UnaryOp")),
        }
    }
}

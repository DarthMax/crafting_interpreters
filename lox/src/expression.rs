use std::fmt;
use std::fmt::Formatter;

use crate::expression::BinaryOp::*;
use crate::expression::Expression::*;
use crate::expression::LiteralType::*;
use crate::expression::UnaryOp::*;

enum Expression {
    Unary {
        inner: Box<Expression>,
        op: UnaryOp,
    },
    Binary {
        left: Box<Expression>,
        right: Box<Expression>,
        op: BinaryOp,
    },
    Literal {
        value: LiteralType,
    },
    Grouping {
        inner: Box<Expression>,
    },
}

impl Expression {
    fn pretty(&self) -> String {
        fn pretty(expr: &Expression, level: u32) -> String {
            let mut prefix = if level == 0 {
                "".to_string()
            } else {
                let mut prefix = "   ".repeat(level as usize);
                prefix.push_str("|_ ");
                prefix
            };

            let thing = match expr {
                Unary { inner, op } => {
                    format!("Unary {}\n{}", op, pretty(inner, level + 1))
                }
                Binary { left, right, op } => {
                    format!(
                        "Binary {}\n{}\n{}",
                        op,
                        pretty(left, level + 1),
                        pretty(right, level + 1)
                    )
                }
                Literal { value } => format!("{}", value),
                Grouping { inner } => {
                    format!("Group\n{}", pretty(inner, level + 1))
                }
                _ => "".to_string(),
            };

            prefix.push_str(&thing);

            prefix
        }

        pretty(self, 0)
    }
}

enum LiteralType {
    NumberLit { value: f64 },
    StringLit { value: String },
    TrueLit,
    FalseLit,
}

impl fmt::Display for LiteralType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NumberLit { value } => write!(f, "{}", value),
            StringLit { value } => write!(f, "\"{}\"", value),
            TrueLit => write!(f, "true"),
            FalseLit => write!(f, "false"),
        }
    }
}

enum BinaryOp {
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

enum UnaryOp {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foo() {
        let expression = Binary {
            left: Box::new(Binary {
                left: Box::new(Literal {
                    value: NumberLit { value: 42f64 },
                }),
                right: Box::new(Binary {
                    left: Box::new(Literal {
                        value: NumberLit { value: 42f64 },
                    }),
                    right: Box::new(Literal {
                        value: NumberLit { value: 42f64 },
                    }),
                    op: Add,
                }),
                op: Add,
            }),
            right: Box::new(Literal {
                value: NumberLit { value: 42f64 },
            }),
            op: Add,
        };

        println!("{}", expression.pretty());
    }
}

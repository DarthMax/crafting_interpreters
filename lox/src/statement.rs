use crate::expression::ExpressionNode;
use crate::token::Token;

pub enum Statement {
    Print(ExpressionNode),
    Expression(ExpressionNode),
    // Var { name: Token, inner: ExpressionNode },
}

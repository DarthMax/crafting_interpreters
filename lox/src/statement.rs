use crate::expression::ExpressionNode;

pub enum Statement {
    Print(ExpressionNode),
    Expression(ExpressionNode),
    // Var { name: Token, inner: ExpressionNode },
}

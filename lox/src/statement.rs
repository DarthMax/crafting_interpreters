use crate::expression::ExpressionNode;

pub enum Statement {
    Print(ExpressionNode),
    Expression(ExpressionNode),
    Var {
        name: String,
        initializer: Option<ExpressionNode>,
    },
}

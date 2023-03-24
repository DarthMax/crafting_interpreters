use crate::expression::ExpressionNode;

pub enum Statement {
    Print(ExpressionNode),
    Expression(ExpressionNode),
    Var {
        name: String,
        initializer: Option<ExpressionNode>,
    },
    Block(Vec<Statement>),
    If {
        condition: ExpressionNode,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
}
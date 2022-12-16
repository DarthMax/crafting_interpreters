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

enum LiteralType {
    NumberLit { value: f64 },
    StringLit { value: String },
    TrueLit,
    FalseLit,
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

enum UnaryOp {
    Not,
    Negative,
}

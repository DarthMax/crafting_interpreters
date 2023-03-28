use std::borrow::Borrow;
use std::iter::Peekable;
use std::slice::Iter;
use std::sync::Arc;

use crate::error::{LoxError, ParseError};
use crate::expression::Expression::{Binary, Call, Grouping, Literal, Logical, Unary, Variable};
use crate::expression::LiteralType::{FalseLit, NilLit, NumberLit, StringLit, TrueLit};
use crate::expression::{BinaryOp, Expression, ExpressionNode, LogicalOp, UnaryOp};
use crate::position::Position;
use crate::statement::Statement;
use crate::token::TokenType::*;
use crate::token::{Token, TokenType};

pub type ParseResult<T> = Result<T, LoxError>;

struct TokenIter<'a> {
    peekable: Peekable<Iter<'a, Token>>,
    size: usize,
}

impl<'a> TokenIter<'a> {
    pub fn new(tokens: &[Token]) -> TokenIter {
        let last_token = tokens.last().unwrap();
        let peekable = tokens.iter().peekable();
        TokenIter {
            peekable,
            size: last_token.position.absolute + last_token.position.length,
        }
    }

    pub fn peek(&mut self) -> Option<&&Token> {
        self.peekable.peek()
    }

    fn next(&mut self) -> Option<&Token> {
        self.peekable.next()
    }

    pub fn next_if(&mut self, func: impl FnOnce(&&Token) -> bool) -> Option<&Token> {
        self.peekable.next_if(func)
    }
}

pub fn parse(tokens: &[Token]) -> ParseResult<Vec<Statement>> {
    let mut token_iter = TokenIter::new(tokens);
    let mut statements = Vec::new();

    while token_iter.peek().is_some() {
        statements.push(declaration(&mut token_iter)?);
    }

    Ok(statements)
}

fn declaration(tokens: &mut TokenIter) -> ParseResult<Statement> {
    match tokens.peek() {
        Some(Token {
            token_type,
            position: _,
        }) => match token_type {
            Fun => {
                let _ = tokens.next();
                function(tokens)
            }
            Var => {
                let _ = tokens.next();
                var(tokens)
            }
            _ => statement(tokens),
        },
        _ => todo!(),
    }
}

fn function(tokens: &mut TokenIter) -> ParseResult<Statement> {
    let name = consume_identifier(tokens)?;

    consume(tokens, LeftParent)?;

    let mut parameters: Vec<String> = vec![];
    if tokens.next_if(|t| t.token_type == RightParent).is_none() {
        loop {
            parameters.push(consume_identifier(tokens)?);

            if tokens.peek().map_or(false, |t| t.token_type == RightParent) {
                tokens.next();
                break;
            }

            consume(tokens, Comma)?;
        }
    }

    let left_brace = consume(tokens, LeftBrace)?;
    let position = left_brace.position.clone();
    let body = block(tokens, position)?;

    Ok(Statement::Function {
        name,
        parameters,
        body: Arc::new(body),
    })
}

fn var(tokens: &mut TokenIter) -> ParseResult<Statement> {
    let identifier = consume_identifier(tokens)?;

    let initializer = match tokens.next_if(|t| t.token_type == Equal) {
        Some(_) => Some(expression(tokens)?),
        None => None,
    };

    consume(tokens, Semicolon)?;

    Ok(Statement::Var {
        name: identifier,
        initializer,
    })
}

fn statement(tokens: &mut TokenIter) -> ParseResult<Statement> {
    match tokens.peek() {
        Some(Token {
            token_type,
            position: _,
        }) => match token_type {
            If => {
                let _ = tokens.next();
                if_statement(tokens)
            }
            While => {
                let _ = tokens.next();
                while_statement(tokens)
            }
            For => {
                let _ = tokens.next();
                for_statement(tokens)
            }
            Return => {
                let _ = tokens.next();
                return_statement(tokens)
            }
            Print => {
                let _ = tokens.next();
                print_statement(tokens)
            }
            LeftBrace => {
                let position = tokens.next().unwrap().position.clone();
                block(tokens, position)
            }
            _ => expression_statement(tokens),
        },
        _ => todo!(),
    }
}

fn if_statement(tokens: &mut TokenIter) -> ParseResult<Statement> {
    let condition = expression(tokens)?;

    let then_branch = statement(tokens)?;
    let else_branch = match tokens.next_if(|t| t.token_type == Else) {
        Some(_) => Some(statement(tokens)?),
        None => None,
    };

    Ok(Statement::If {
        condition,
        then_branch: Box::new(then_branch),
        else_branch: else_branch.map(Box::new),
    })
}

fn while_statement(tokens: &mut TokenIter) -> ParseResult<Statement> {
    let condition = expression(tokens)?;
    let body = statement(tokens)?;

    Ok(Statement::While {
        condition,
        body: Box::new(body),
    })
}

fn for_statement(tokens: &mut TokenIter) -> ParseResult<Statement> {
    let initializer = match tokens.peek() {
        Some(Token {
            token_type: Var,
            position: _,
        }) => {
            let _ = tokens.next();
            Some(var(tokens)?)
        }
        Some(Token {
            token_type: Semicolon,
            position: _,
        }) => {
            let _ = tokens.next();
            None
        }
        Some(_) => Some(expression_statement(tokens)?),
        None => {
            return Err(ParseError::unexpected_end_of_stream());
        }
    };

    fn parse(tokens: &mut TokenIter) -> ParseResult<Option<ExpressionNode>> {
        match tokens.peek() {
            Some(Token {
                token_type: Semicolon,
                position: _,
            }) => {
                let _ = tokens.next();
                Ok(None)
            }
            Some(_) => {
                let expr = Ok(Some(expression(tokens)?));
                consume(tokens, Semicolon)?;
                expr
            }
            None => Err(ParseError::unexpected_end_of_stream()),
        }
    }

    let condition = parse(tokens)?;
    let increment = parse(tokens)?;

    let mut body = statement(tokens)?;

    if let Some(increment) = increment {
        body = Statement::Block(vec![body, Statement::Expression(increment)]);
    }

    body = Statement::While {
        condition: condition.unwrap_or(ExpressionNode::new(Literal(TrueLit), &Position::new(0, 1))),
        body: Box::new(body),
    };

    if let Some(initializer) = initializer {
        body = Statement::Block(vec![initializer, body]);
    }

    Ok(body)
}

fn return_statement(tokens: &mut TokenIter) -> ParseResult<Statement> {
    let return_expression = match tokens.peek() {
        Some(Token {
            token_type: Semicolon,
            position: _,
        }) => {
            tokens.next();
            None
        }
        _ => Some(expression(tokens)?),
    };

    consume(tokens, Semicolon)?;

    Ok(Statement::Return(return_expression))
}

fn print_statement(tokens: &mut TokenIter) -> ParseResult<Statement> {
    let expression = expression(tokens)?;
    let _ = consume(tokens, Semicolon)?;
    Ok(Statement::Print(expression))
}

fn block(tokens: &mut TokenIter, opening_brace_pos: Position) -> ParseResult<Statement> {
    let mut statements = Vec::new();

    loop {
        match tokens.peek() {
            Some(Token {
                token_type: RightBrace,
                position: _,
            }) => {
                break;
            }
            Some(_) => {}
            None => {
                return Err(ParseError::unexpected_end_of_stream());
            }
        }

        statements.push(declaration(tokens)?);
    }

    let _ = consume_closing_delimiter(tokens, RightBrace, &opening_brace_pos)?;

    Ok(Statement::Block(statements))
}

fn expression_statement(tokens: &mut TokenIter) -> ParseResult<Statement> {
    let expression = expression(tokens)?;
    let _ = consume(tokens, Semicolon)?;
    Ok(Statement::Expression(expression))
}

fn expression(tokens: &mut TokenIter) -> ParseResult<ExpressionNode> {
    assignment(tokens)
}

fn assignment(tokens: &mut TokenIter) -> ParseResult<ExpressionNode> {
    let expression = or(tokens)?;

    match tokens.next_if(|n| n.token_type == Equal) {
        Some(_) => {
            let value = assignment(tokens)?;
            match expression.expression {
                Variable(name) => {
                    let length = value.position.end_position() - expression.position.absolute;
                    let assignment = Expression::Assignment {
                        name,
                        value: Box::new(value),
                    };

                    let position = Position::new(expression.position.absolute, length);

                    Ok(ExpressionNode::new(assignment, &position))
                }
                _ => Err(ParseError::invalid_assignment_target(&expression.position)),
            }
        }
        None => Ok(expression),
    }
}

fn or(tokens: &mut TokenIter) -> ParseResult<ExpressionNode> {
    parse_logical_op(tokens, &[Or], and)
}

fn and(tokens: &mut TokenIter) -> ParseResult<ExpressionNode> {
    parse_logical_op(tokens, &[And], equality)
}

fn equality(tokens: &mut TokenIter) -> ParseResult<ExpressionNode> {
    parse_binary_op(tokens, &[BangEqual, EqualEqual], comparison)
}

fn comparison(tokens: &mut TokenIter) -> ParseResult<ExpressionNode> {
    parse_binary_op(tokens, &[Greater, GreaterEqual, Less, LessEqual], term)
}

fn term(tokens: &mut TokenIter) -> ParseResult<ExpressionNode> {
    parse_binary_op(tokens, &[Minus, Plus], factor)
}

fn factor(tokens: &mut TokenIter) -> ParseResult<ExpressionNode> {
    parse_binary_op(tokens, &[Slash, Star], unary)
}

fn unary(tokens: &mut TokenIter) -> ParseResult<ExpressionNode> {
    match tokens.next_if(|token| token.token_type == Bang || token.token_type == Minus) {
        Some(Token {
            token_type,
            position,
            ..
        }) => {
            let op: UnaryOp = token_type.try_into().unwrap();
            let position = position.clone();

            let inner = unary(tokens)?;
            let expression = Unary {
                inner: Box::new(inner),
                op,
            };
            Ok(ExpressionNode::new(expression, &position))
        }
        _ => call(tokens),
    }
}

fn call(tokens: &mut TokenIter) -> ParseResult<ExpressionNode> {
    let mut expr = primary(tokens)?;

    while let Some(t) = tokens.next_if(|t| t.token_type == LeftParent) {
        let mut position = t.position.clone();

        let mut arguments: Vec<ExpressionNode> = vec![];

        if tokens.next_if(|t| t.token_type == RightParent).is_none() {
            loop {
                let argument = expression(tokens)?;
                position.union(&argument.position);
                arguments.push(argument);

                if tokens.peek().map_or(false, |t| t.token_type == RightParent) {
                    position.union(&tokens.next().unwrap().position);
                    break;
                }

                position.union(&consume(tokens, Comma)?.position);
            }
        }

        expr = ExpressionNode::raw(
            Call {
                callee: Box::new(expr),
                arguments,
            },
            position,
        );
    }

    Ok(expr)
}

fn primary(tokens: &mut TokenIter) -> ParseResult<ExpressionNode> {
    match tokens.next() {
        Some(token) => {
            let mut position = token.position.clone();
            let expression = match &token.token_type {
                False => Ok(Literal(FalseLit)),
                True => Ok(Literal(TrueLit)),
                Nil => Ok(Literal(NilLit)),
                Number(value) => Ok(Literal(NumberLit(*value))),
                StringToken(value) => Ok(Literal(StringLit(value.clone()))),
                LeftParent => {
                    let inner = expression(tokens)?;
                    let end_position = consume_closing_delimiter(tokens, RightParent, &position)?
                        .position
                        .clone();

                    position = Position::new(
                        position.absolute,
                        end_position.end_position() - position.absolute,
                    );

                    Ok(Grouping(Box::new(inner)))
                }
                Identifier(identifier) => Ok(Variable(identifier.to_string())),
                _ => Err(ParseError::illegal_token((*token).clone())),
            };
            Ok(ExpressionNode::new(expression?, &position))
        }
        None => Err(ParseError::unexpected_end_of_stream()),
    }
}

fn parse_logical_op(
    tokens: &mut TokenIter,
    accepted_token_types: &[TokenType],
    inner_parser: fn(&mut TokenIter) -> ParseResult<ExpressionNode>,
) -> ParseResult<ExpressionNode> {
    parse_bi_op(
        tokens,
        accepted_token_types,
        inner_parser,
        |left, right, op: LogicalOp| Logical { left, right, op },
    )
}

fn parse_binary_op(
    tokens: &mut TokenIter,
    accepted_token_types: &[TokenType],
    inner_parser: fn(&mut TokenIter) -> ParseResult<ExpressionNode>,
) -> ParseResult<ExpressionNode> {
    parse_bi_op(
        tokens,
        accepted_token_types,
        inner_parser,
        |left, right, op: BinaryOp| Binary { left, right, op },
    )
}

fn parse_bi_op<OpType>(
    tokens: &mut TokenIter,
    accepted_token_types: &[TokenType],
    inner_parser: fn(&mut TokenIter) -> ParseResult<ExpressionNode>,
    expression_creator: fn(Box<ExpressionNode>, Box<ExpressionNode>, OpType) -> Expression,
) -> ParseResult<ExpressionNode>
where
    for<'a> OpType: TryFrom<&'a TokenType>,
{
    let mut expression_node = inner_parser(tokens)?;

    loop {
        let maybe_op_token =
            tokens.next_if(|token| accepted_token_types.contains(&token.token_type));

        match maybe_op_token {
            Some(op_token) => {
                let op: OpType = match op_token.token_type.borrow().try_into() {
                    Ok(op) => op,
                    Err(_) => {
                        return Err(ParseError::illegal_token(op_token.clone()));
                    }
                };

                let left = Box::new(expression_node);
                let right = Box::new(inner_parser(tokens)?);

                let start_pos = left.position.absolute;
                let length = right.position.absolute + right.position.length - start_pos;

                let expression = expression_creator(left, right, op);

                expression_node = ExpressionNode::new(
                    expression,
                    &Position {
                        absolute: start_pos,
                        length,
                    },
                )
            }
            None => break,
        }
    }

    Ok(expression_node)
}

fn consume_identifier(tokens: &mut TokenIter) -> Result<String, LoxError> {
    let matcher = |token: &TokenType| matches!(token, Identifier(_));
    let expected = "Identifier".to_string();

    let identifier = _consume(tokens, matcher, expected, || {
        ParseError::unexpected_end_of_stream()
    })?;

    match &identifier.token_type {
        Identifier(i) => Ok(i.clone()),
        _ => panic!(),
    }
}

fn consume_closing_delimiter<'a>(
    tokens: &'a mut TokenIter,
    expected: TokenType,
    opening_delimiter_position: &Position,
) -> Result<&'a Token, LoxError> {
    let matcher = |t: &TokenType| *t == expected;
    let expected = expected.to_string();
    let eof_pos = tokens.size;
    let eof_error =
        || ParseError::unclosed_delimiter(opening_delimiter_position, &Position::new(eof_pos, 1));

    _consume(tokens, matcher, expected, eof_error)
}

fn consume<'a>(tokens: &'a mut TokenIter, token_type: TokenType) -> Result<&'a Token, LoxError> {
    let matcher = |t: &TokenType| *t == token_type;
    let expected = token_type.to_string();
    let eof_pos = tokens.size;
    let eof_error = || ParseError::unexpected_token_raw(Eof, Semicolon, Position::new(eof_pos, 1));

    _consume(tokens, matcher, expected, eof_error)
}

fn _consume<'a, TokenMatcher, ErrorFn>(
    tokens: &'a mut TokenIter,
    token_matcher: TokenMatcher,
    expected: String,
    eof_error: ErrorFn,
) -> Result<&'a Token, LoxError>
where
    TokenMatcher: Fn(&TokenType) -> bool,
    ErrorFn: Fn() -> LoxError,
{
    match tokens.next() {
        Some(t @ Token { token_type, .. }) if token_matcher(token_type) => Ok(t),
        Some(token) => Err(ParseError::unexpected_token(token.clone(), expected)),
        None => Err(eof_error()),
    }
}

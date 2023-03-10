use std::iter::Peekable;
use std::slice::Iter;

use crate::error::{LoxError, ParseError};
use crate::expression::Expression::{Binary, Grouping, Literal, Unary};
use crate::expression::ExpressionNode;
use crate::expression::LiteralType::{FalseLit, NilLit, NumberLit, StringLit, TrueLit};
use crate::position::Position;
use crate::statement::Statement;
use crate::token::TokenType::*;
use crate::token::{Token, TokenType};

pub type TokenIter<'a> = Peekable<Iter<'a, Token>>;
pub type ParseResult<T> = Result<T, LoxError>;

pub fn parse(tokens: &[Token]) -> ParseResult<Vec<Statement>> {
    let mut token_iter: TokenIter = tokens.iter().peekable();
    let mut statements = Vec::new();

    while token_iter.peek().is_some() {
        statements.push(statement(&mut token_iter)?);
    }

    Ok(statements)
}

fn statement(tokens: &mut TokenIter) -> ParseResult<Statement> {
    match tokens.peek() {
        Some(Token {
            token_type,
            position: _,
        }) => match token_type {
            Print => {
                let _ = tokens.next();
                print_statement(tokens)
            }
            _ => expression_statement(tokens),
        },
        _ => todo!(),
    }
}

fn print_statement(tokens: &mut TokenIter) -> ParseResult<Statement> {
    let expression = expression(tokens)?;
    let _ = consume_semicolon(tokens, Semicolon, &expression.position)?;
    Ok(Statement::Print(expression))
}

fn expression_statement(tokens: &mut TokenIter) -> ParseResult<Statement> {
    let expression = expression(tokens)?;
    let _ = consume_semicolon(tokens, Semicolon, &expression.position)?;
    Ok(Statement::Expression(expression))
}

fn expression(tokens: &mut TokenIter) -> ParseResult<ExpressionNode> {
    equality(tokens)
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
            let inner = unary(tokens)?;
            let expression = Unary {
                inner: Box::new(inner),
                op: token_type.try_into().unwrap(),
            };
            Ok(ExpressionNode::new(expression, position))
        }
        _ => primary(tokens),
    }
}

fn primary(tokens: &mut TokenIter) -> ParseResult<ExpressionNode> {
    match tokens.next() {
        Some(token) => {
            let mut position = token.position.clone();
            let expression = match &token.token_type {
                False => Ok(Literal(FalseLit)),
                True => Ok(Literal(TrueLit)),
                Nil => Ok(Literal(NilLit)),
                Number { value } => Ok(Literal(NumberLit(*value))),
                StringToken { value } => Ok(Literal(StringLit(value.clone()))),
                LeftParent => {
                    let inner = expression(tokens)?;
                    let end_position = consume_closing_delimiter(
                        tokens,
                        RightParent,
                        &token.position,
                        &inner.position,
                    )?;

                    position = Position::new(
                        position.absolute,
                        end_position.end_position() - position.absolute,
                    );

                    Ok(Grouping(Box::new(inner)))
                }
                _ => Err(ParseError::illegal_token((*token).clone())),
            };
            // TODO! position here is wrong
            Ok(ExpressionNode::new(expression?, &position))
        }
        None => Err(ParseError::unexpected_end_of_stream()),
    }
}

fn parse_binary_op(
    tokens: &mut TokenIter,
    accepted_token_types: &[TokenType],
    inner_parser: fn(&mut TokenIter) -> ParseResult<ExpressionNode>,
) -> ParseResult<ExpressionNode> {
    let mut expression_node = inner_parser(tokens)?;

    loop {
        let maybe_op_token =
            tokens.next_if(|token| accepted_token_types.contains(&token.token_type));

        match maybe_op_token {
            Some(op_token) => {
                let left = Box::new(expression_node);
                let right = Box::new(inner_parser(tokens)?);

                let start_pos = left.position.absolute;
                let length = right.position.absolute + right.position.length - start_pos;

                let expression = Binary {
                    left,
                    right,
                    op: (&op_token.token_type).try_into().unwrap(),
                };

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

fn consume_semicolon(
    tokens: &mut TokenIter,
    expected: TokenType,
    prev_position: &Position,
) -> Result<Position, LoxError> {
    consume(tokens, expected, || {
        ParseError::unexpected_token_raw(
            Eof,
            Semicolon,
            Position::new(prev_position.absolute + prev_position.length, 1),
        )
    })
}

fn consume_closing_delimiter(
    tokens: &mut TokenIter,
    expected: TokenType,
    opening_delimiter_position: &Position,
    prev_position: &Position,
) -> Result<Position, LoxError> {
    consume(tokens, expected, || {
        ParseError::unclosed_delimiter(
            opening_delimiter_position,
            &Position::new(prev_position.absolute + prev_position.length, 1),
        )
    })
}

fn consume<F>(
    tokens: &mut TokenIter,
    expected: TokenType,
    eof_error: F,
) -> Result<Position, LoxError>
where
    F: Fn() -> LoxError,
{
    match tokens.next() {
        Some(t @ Token { token_type, .. }) if *token_type == expected => Ok(t.position.clone()),
        Some(token) => Err(ParseError::unexpected_token(token.clone(), expected)),
        None => Err(eof_error()),
    }
}

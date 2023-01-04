use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::slice::Iter;

use crate::expression::Expression;
use crate::expression::Expression::{Binary, Grouping, Literal, Unary};
use crate::expression::LiteralType::{FalseLit, NillLit, NumberLit, StringLit, TrueLit};
use crate::parser::ParseError::{UnexpectedEndOfTokenStream, UnexpectedToken};
use crate::scanner::TokenType::*;
use crate::scanner::{Token, TokenType};

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        found: Token,
        expected: Option<TokenType>,
    },
    UnexpectedEndOfTokenStream,
}

impl ParseError {
    pub fn unexpected_token(found: Token, expected: Option<TokenType>) -> ParseError {
        UnexpectedToken { found, expected }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse Error: ")?;

        match self {
            UnexpectedToken {
                found,
                expected: None,
            } => write!(
                f,
                "Unexpected token: {} at {}:{}",
                found.token_type, found.line, found.column
            ),
            UnexpectedToken {
                found,
                expected: Some(expected),
            } => write!(
                f,
                "Unexpected token `{}` expected `{}` at {}:{}",
                found.token_type, expected, found.line, found.column
            ),
            UnexpectedEndOfTokenStream => write!(f, "Unexpected end of token stream"),
        }?;

        Ok(())
    }
}

pub type TokenIter<'a> = Peekable<Iter<'a, Token>>;
pub type ParseResult = Result<Expression, ParseError>;

pub fn parse(tokens: &Vec<Token>) -> ParseResult {
    let mut token_iter: TokenIter = tokens.into_iter().peekable();

    expression(&mut token_iter)
}

fn expression(tokens: &mut TokenIter) -> ParseResult {
    equality(tokens)
}

fn equality(tokens: &mut TokenIter) -> ParseResult {
    parse_binary_op(tokens, &[BangEqual, EqualEqual], comparison)
}

fn comparison(tokens: &mut TokenIter) -> ParseResult {
    parse_binary_op(tokens, &[Greater, GreaterEqual, Less, LessEqual], term)
}

fn term(tokens: &mut TokenIter) -> ParseResult {
    parse_binary_op(tokens, &[Minus, Plus], factor)
}

fn factor(tokens: &mut TokenIter) -> ParseResult {
    parse_binary_op(tokens, &[Slash, Star], unary)
}

fn unary(tokens: &mut TokenIter) -> ParseResult {
    match tokens.next_if(|token| token.token_type == Bang || token.token_type == Minus) {
        Some(Token { token_type, .. }) => unary(tokens).map(|inner_expression| Unary {
            inner: Box::new(inner_expression),
            op: token_type.try_into().unwrap(),
        }),
        _ => primary(tokens),
    }
}

fn primary(tokens: &mut TokenIter) -> ParseResult {
    match tokens.next() {
        Some(token) => match &token.token_type {
            False => Ok(Literal { value: FalseLit }),
            True => Ok(Literal { value: TrueLit }),
            Nil => Ok(Literal { value: NillLit }),
            Number { value } => Ok(Literal {
                value: NumberLit { value: *value },
            }),
            StringToken { value } => Ok(Literal {
                value: StringLit {
                    value: value.clone(),
                },
            }),
            LeftParent => match expression(tokens) {
                Ok(inner) => consume(tokens, RightParent).map(|_| Grouping {
                    inner: Box::new(inner),
                }),
                e => e,
            },
            _ => Err(ParseError::unexpected_token((*token).clone(), None)),
        },
        None => Err(UnexpectedEndOfTokenStream),
    }
}

fn parse_binary_op(
    tokens: &mut TokenIter,
    accepted_token_types: &[TokenType],
    inner_parser: fn(&mut TokenIter) -> ParseResult,
) -> ParseResult {
    let mut expression = inner_parser(tokens);

    while expression.is_ok() {
        let maybe_op_token =
            tokens.next_if(|token| accepted_token_types.contains(&token.token_type));

        match maybe_op_token {
            Some(op_token) => {
                expression = inner_parser(tokens).map(|right_expr| Binary {
                    left: Box::new(expression.unwrap()),
                    right: Box::new(right_expr),
                    op: (&op_token.token_type).try_into().unwrap(),
                })
            }
            None => break,
        }
    }

    expression
}

fn consume(tokens: &mut TokenIter, expected: TokenType) -> Result<(), ParseError> {
    match tokens.peek() {
        Some(Token { token_type, .. }) if *token_type == expected => {
            tokens.next();
            Ok(())
        }
        Some(token) => Err(ParseError::unexpected_token(
            (**token).clone(),
            Some(expected),
        )),
        None => Err(UnexpectedEndOfTokenStream),
    }
}

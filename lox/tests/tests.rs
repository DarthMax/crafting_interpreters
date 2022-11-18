use lox::{Scanner, Token, TokenType};
use TokenType::*;

fn run_test(content: &str, expected: Vec<Token>) {
    use pretty_assertions::assert_eq;

    let scanner = Scanner::new(content.to_string());
    let actual = scanner.scan();

    assert_eq!(actual, expected);
}

fn gen_test(content: &str) {
    use pretty_assertions::assert_eq;

    let scanner = Scanner::new(content.to_string());
    let actual = scanner.scan();

    actual.into_iter().for_each(|t| println!("{:?}", t));
}

#[test]
fn test_classes() {
    run_test(include_str!("classes.lox"), include!("classes.tokens"))
}

#[test]
fn test_control_flow() {
    run_test(
        include_str!("control_flow.lox"),
        include!("control_flow.tokens"),
    )
}

#[test]
fn test_expressions() {
    run_test(
        include_str!("expressions.lox"),
        include!("expressions.tokens"),
    )
}

#[test]
fn test_functions() {
    run_test(include_str!("functions.lox"), include!("functions.tokens"))
}

#[test]
fn test_hello_world() {
    run_test(
        include_str!("hello_world.lox"),
        include!("hello_world.tokens"),
    )
}

#[test]
fn test_statements() {
    run_test(
        include_str!("statements.lox"),
        include!("statements.tokens"),
    )
}

#[test]
fn test_types() {
    run_test(include_str!("types.lox"), include!("types.tokens"))
}

#[test]
fn test_variables() {
    run_test(include_str!("variables.crox"), include!("variables.tokens"))
}

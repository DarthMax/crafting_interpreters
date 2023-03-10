extern crate core;

use std::ffi::OsString;
use std::io::Write;
use std::{env, fs, io};

use crate::evaluation::evaluate;
use crate::scanner::Scanner;

mod error;
mod evaluation;
mod expression;
mod parser;
mod position;
mod scanner;
mod statement;
mod token;

fn main() {
    let mut args = env::args_os().skip(1);
    let file = args.next();

    if args.next().is_some() {
        println!("Usage: lox [script]");
        std::process::exit(64);
    }

    let result = match file {
        Some(file) => run_file(file),
        None => run_repl(),
    };

    match result {
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1)
        }
        _ => std::process::exit(0),
    }
}

fn run_file(file: OsString) -> io::Result<()> {
    let source = fs::read_to_string(file)?;
    parse(source);
    Ok(())
}

fn run_repl() -> io::Result<()> {
    let mut line_number = 0_u32;
    let mut input = String::new();

    loop {
        print!("lox ({line_number})> ");
        io::stdout().flush()?;

        input.clear();
        io::stdin().read_line(&mut input)?;

        if input.trim().eq("quit") {
            println!("Good bye!");
            break;
        }

        parse(input.clone());

        line_number += 1;
    }

    Ok(())
}

fn parse(source: String) {
    let scanner = Scanner::new(source.clone());
    let tokens = scanner.scan();
    match parser::parse(&tokens) {
        Ok(expression) => match evaluate(&expression) {
            Ok(value) => println!("{value:?}"),
            Err(error) => println!("{:?}", miette::Report::new(error).with_source_code(source)),
        },
        Err(error) => println!("{:?}", miette::Report::new(error).with_source_code(source)),
    };
}

extern crate core;

use std::borrow::Cow;
use std::cell::RefCell;
use std::ffi::OsString;
use std::rc::Rc;
use std::{env, fs, io};

use reedline::{
    default_emacs_keybindings, EditCommand, Emacs, KeyCode, KeyModifiers, Prompt, PromptEditMode,
    PromptHistorySearch, PromptHistorySearchStatus, Reedline, ReedlineEvent, Signal,
};

use crate::environment::Environment;
use crate::evaluation::evaluate;
use crate::scanner::Scanner;

mod callable;
mod environment;
mod error;
mod evaluation;
mod expression;
mod parser;
mod position;
mod scanner;
mod statement;
mod token;
mod value;

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
    let env = Rc::new(RefCell::new(Environment::empty()));

    run(source, env);
    Ok(())
}

fn run_repl() -> io::Result<()> {
    let mut line_editor = create_repl();
    let mut prompt = ReplPrompt { line: 0 };

    let env = Rc::new(RefCell::new(Environment::empty()));

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(buffer)) => {
                run(buffer, env.clone());
            }
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                println!("\nGood Bye!");
                break;
            }
            _ => todo!(),
        }
        prompt.line += 1;
    }

    Ok(())
}

fn run(source: String, env: Rc<RefCell<Environment>>) {
    let scanner = Scanner::new(source.clone());
    let tokens = scanner.scan();
    match parser::parse(&tokens) {
        Ok(expression) => match evaluate(&expression, env) {
            Ok(value) => println!("{value:?}"),
            Err(error) => println!("{:?}", miette::Report::new(error).with_source_code(source)),
        },
        Err(error) => println!("{:?}", miette::Report::new(error).with_source_code(source)),
    };
}

fn create_repl() -> Reedline {
    let mut keybindings = default_emacs_keybindings();

    keybindings.add_binding(
        KeyModifiers::ALT,
        KeyCode::Enter,
        ReedlineEvent::Edit(vec![EditCommand::InsertNewline]),
    );

    Reedline::create().with_edit_mode(Box::new(Emacs::new(keybindings)))
}

struct ReplPrompt {
    line: usize,
}

impl Prompt for ReplPrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        Cow::Owned(format!("lox:{}", self.line))
    }

    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Owned("".to_string())
    }

    fn render_prompt_indicator(&self, _: PromptEditMode) -> Cow<str> {
        Cow::Owned("> ".to_string())
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Owned(format!("...:{}> ", self.line))
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        // NOTE: magic strings, given there is logic on how these compose I am not sure if it
        // is worth extracting in to static constant
        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ))
    }
}

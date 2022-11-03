use std::env;
mod scanner;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        
        println!("REPL")
    } else if args.len() == 2 {
        println!("From file")
    } else {
        println!("error")
    }
}

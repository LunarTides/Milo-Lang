use std::{
    fs,
    io::{self, Write},
};

use clap::Parser as _;
use lexer::Lexer;
use parser::Parser;

mod lexer;
mod parser;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(required_unless_present("repl"), trailing_var_arg(true))]
    file_path: Vec<String>,

    #[arg(short, long)]
    repl: bool,
}

fn read_code_from_file(file_path: String) -> String {
    fs::read_to_string(file_path).expect("Please pass a valid file")
}

fn main() {
    let args = Args::parse();

    if args.repl {
        let mut lexer = Lexer::default();
        let mut parser = Parser::default();

        loop {
            let mut input = String::new();

            print!("> ");
            io::stdout().flush().unwrap();

            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let tokens = lexer.lex_code(input);
                    parser.parse(tokens);
                    parser.should_abort = false;
                }
                Err(err) => panic!("{}", err),
            };
        }
    }

    let mut lexer = Lexer::default();
    let mut parser = Parser::default();

    let code = read_code_from_file(args.file_path[0].clone());

    let tokens = lexer.lex_code(code);
    parser.parse(tokens);
}

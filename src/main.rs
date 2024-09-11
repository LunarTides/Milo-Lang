use std::{env, fs};

use lexer::Lexer;
use parser::Parser;

mod lexer;
mod parser;

fn read_code_from_file(file_path: String) -> String {
    fs::read_to_string(file_path).expect("Please pass a valid file")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Please supply a valid path. E.g. <path to executable> ./example.milo")
    }

    let code = read_code_from_file(args[1].clone());

    let mut lexer = Lexer::default();
    let tokens = lexer.lex_code(code);

    let mut parser = Parser::new(tokens);
    parser.parse();
}

use crate::lexer::{LexedTokens, Token};

#[derive(Default)]
pub struct Parser {
    stack: Vec<String>,
}

impl Parser {
    pub fn parse(&mut self, all_tokens: LexedTokens) {
        println!(
            "--- Tokens ---\n{:?}\n--------------\n\n--- Output ---",
            all_tokens
        );

        for mut tokens in all_tokens {
            tokens.reverse();

            for token in tokens {
                match token.1 {
                    Token::Identifier => self.parse_identifier(token.0),
                    Token::Number | Token::String => self.stack.push(token.0),
                }
            }
        }

        println!("--------------");
    }

    fn parse_identifier(&mut self, identifier: String) {
        if identifier == "print" {
            println!("{}", self.stack.last().unwrap());
            self.stack.pop();
        } else if identifier == "add" {
            let a = self.stack.pop().unwrap().parse::<i64>().unwrap();
            let b = self.stack.pop().unwrap().parse::<i64>().unwrap();

            self.stack.push((a + b).to_string());
        }
    }
}

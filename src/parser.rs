use crate::lexer::{LexedToken, LexedTokens, Token};

#[derive(Default)]
pub struct Parser {
    stack: Vec<LexedToken>,
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
                    Token::Number | Token::String => self.stack.push(token),
                }
            }
        }

        println!("--------------");
    }

    fn parse_identifier(&mut self, identifier: String) {
        if identifier == "print" {
            let token = self.pop_token("print", 1, Some(("".into(), Token::String)));
            let mut to_print = &token.0;

            // Remove leading zeroes if the value is a number.
            let to_print_if_number = &to_print.trim_start_matches('0').to_string();

            if token.1 == Token::Number {
                to_print = to_print_if_number
            }

            println!("{}", to_print);
        } else if identifier == "add" {
            let a = self.pop_token("add", 2, None).0.parse::<i64>().unwrap();
            let b = self.pop_token("add", 2, None).0.parse::<i64>().unwrap();

            self.stack.push(((a + b).to_string(), Token::Number));
        }
    }

    fn pop_token(
        &mut self,
        identifier: &str,
        expected_argument_amount: u8,
        default: Option<LexedToken>,
    ) -> LexedToken {
        self.stack.pop().or(default).unwrap_or_else(|| {
            panic!(
                "`{}` needs {} argument(s).",
                identifier, expected_argument_amount
            )
        })
    }
}

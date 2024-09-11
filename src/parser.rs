use crate::lexer::{LexedTokenLines, Token, TokenType};

#[derive(Default)]
pub struct Parser {
    stack: Vec<Token>,
}

impl Parser {
    pub fn parse(&mut self, all_tokens: LexedTokenLines) {
        println!(
            "--- Tokens ---\n{:?}\n--------------\n\n--- Output ---",
            all_tokens
        );

        for mut tokens in all_tokens {
            tokens.reverse();

            for token in tokens {
                match token.token_type {
                    TokenType::Identifier => self.parse_identifier(token.value),
                    TokenType::Number | TokenType::String => self.stack.push(token),
                }
            }
        }

        println!("--------------");
    }

    fn parse_identifier(&mut self, identifier: String) {
        if identifier == "print" {
            let token = self.pop_token(
                "print",
                1,
                Some(Token {
                    token_type: TokenType::String,
                    value: "".to_string(),
                }),
            );

            let mut to_print = &token.value;

            // Remove leading zeroes if the value is a number.
            let to_print_if_number = &to_print.trim_start_matches('0').to_string();

            if token.token_type == TokenType::Number {
                to_print = to_print_if_number
            }

            println!("{}", to_print);
        } else if identifier == "add" {
            let a = self.pop_token("add", 2, None).value.parse::<i64>().unwrap();
            let b = self.pop_token("add", 2, None).value.parse::<i64>().unwrap();

            self.stack.push(Token {
                token_type: TokenType::Number,
                value: (a + b).to_string(),
            });
        }
    }

    fn pop_token(
        &mut self,
        identifier: &str,
        expected_argument_amount: u8,
        default: Option<Token>,
    ) -> Token {
        self.stack.pop().or(default).unwrap_or_else(|| {
            panic!(
                "`{}` needs {} argument(s).",
                identifier, expected_argument_amount
            )
        })
    }
}

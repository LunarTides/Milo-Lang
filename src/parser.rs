use crate::lexer::{LexedTokens, Token};

#[derive(Default)]
pub struct Parser {
    working_value: String,
}

impl Parser {
    pub fn parse(&mut self, all_tokens: LexedTokens) {
        println!("{:?}", all_tokens);

        for mut tokens in all_tokens {
            tokens.reverse();

            for token in tokens {
                match token.1 {
                    Token::Identifier => self.parse_identifier(token.0),
                    Token::Number {} => self.working_value = token.0.to_string(),
                    Token::String {} => self.working_value = token.0,
                }
            }
        }
    }

    fn parse_identifier(&mut self, identifier: String) {
        if identifier == "print" {
            println!("{}", self.working_value);
            self.working_value = "".into();
        }
    }
}

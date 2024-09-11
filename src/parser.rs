use std::collections::HashMap;

use crate::lexer::{LexedTokenLines, Token, TokenType};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum VariableType {
    #[default]
    String,
    Number,
}

impl From<VariableType> for TokenType {
    fn from(variable_type: VariableType) -> TokenType {
        match variable_type {
            VariableType::String => TokenType::String,
            VariableType::Number => TokenType::Number,
        }
    }
}

struct Variable {
    variable_type: VariableType,
    value: String,
}

#[derive(Default)]
pub struct Parser {
    stack: Vec<Token>,
    variables: HashMap<String, Variable>,
    tokens_in_file: LexedTokenLines,
    tokens_on_line: Vec<Token>,
    line: usize,
    current_token: Token,
    index: usize,
    skip_next: bool,
}

impl Parser {
    pub fn new(tokens: LexedTokenLines) -> Self {
        Parser {
            tokens_in_file: tokens,
            ..Default::default()
        }
    }

    pub fn parse(&mut self) {
        println!(
            "--- Tokens ---\n{:?}\n--------------\n\n--- Output ---",
            self.tokens_in_file
        );

        for (line, mut tokens) in self.tokens_in_file.clone().into_iter().enumerate() {
            self.tokens_on_line.clone_from(&tokens);
            tokens.reverse();

            self.line = line;

            for (i, token) in tokens.clone().into_iter().enumerate() {
                if self.skip_next {
                    self.skip_next = false;
                    continue;
                }

                self.current_token = token.clone();
                self.index = (tokens.len() - i) - 1;

                match token.token_type {
                    TokenType::Identifier => self.parse_identifier(token.value),
                    TokenType::Operator => self.parse_operator(token.value),
                    TokenType::Number | TokenType::String => self.stack.push(token),
                }
            }
        }

        println!("--------------");
    }

    fn parse_identifier(&mut self, identifier: String) {
        if identifier == "print" {
            let token = self.pop_stack(
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
        }

        let var = self.try_parse_variable(&identifier);
        if let Some(variable) = var {
            self.stack.push(variable);
        }
    }

    fn parse_operator(&mut self, operator: String) {
        if operator == "+" {
            let a = self
                .previous_token("+", 2, None)
                .value
                .parse::<i64>()
                .unwrap();
            let b = self.stack.pop().unwrap().value.parse::<i64>().unwrap();

            self.skip_next = true;

            self.stack.push(Token {
                token_type: TokenType::Number,
                value: (a + b).to_string(),
            })
        } else if operator == "=" {
            let name = self.previous_token("=", 2, None);
            let value = self.next_token("=", 2, None);

            self.skip_next = true;

            // TODO: Actually get the variable type.
            self.variables.insert(
                name.value,
                Variable {
                    variable_type: VariableType::Number,
                    value: value.value,
                },
            );
        }
    }

    fn next_token(
        &self,
        identifier: &str,
        expected_argument_amount: u8,
        default: Option<Token>,
    ) -> Token {
        if self.tokens_on_line.len() <= self.index + 1 {
            return default.unwrap_or_else(|| {
                panic!(
                    "`{}` needs {} argument(s).",
                    identifier, expected_argument_amount
                )
            });
        }

        let to_return = self.tokens_on_line[self.index + 1].clone();

        let var = self.try_parse_variable(&to_return.value);
        if let Some(variable) = var {
            return variable;
        }

        to_return
    }

    fn previous_token(
        &self,
        identifier: &str,
        expected_argument_amount: u8,
        default: Option<Token>,
    ) -> Token {
        if self.index == 0 {
            return default.unwrap_or_else(|| {
                panic!(
                    "`{}` needs {} argument(s).",
                    identifier, expected_argument_amount
                )
            });
        }

        let to_return = self.tokens_on_line[self.index - 1].clone();

        let var = self.try_parse_variable(&to_return.value);
        if let Some(variable) = var {
            return variable;
        }

        to_return
    }

    fn try_parse_variable(&self, identifier: &String) -> Option<Token> {
        let var: Option<&Variable> = self.variables.get(identifier);
        if let Some(variable) = var {
            return Some(Token {
                token_type: TokenType::from(variable.variable_type),
                value: variable.value.clone(),
            });
        }

        None
    }

    fn pop_stack(
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

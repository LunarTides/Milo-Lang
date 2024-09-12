use std::{
    collections::HashMap,
    io::{self, Write},
};

use crate::lexer::{LexedTokenLines, Token, TokenType};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum VariableType {
    #[default]
    String,
    Number,
    Boolean,
}

impl From<VariableType> for TokenType {
    fn from(variable_type: VariableType) -> TokenType {
        match variable_type {
            VariableType::String => TokenType::String,
            VariableType::Number => TokenType::Number,
            VariableType::Boolean => TokenType::Boolean,
        }
    }
}

#[derive(Debug)]
struct Variable {
    variable_type: VariableType,
    value: String,
}

#[derive(Default)]
pub struct Parser {
    stack: Vec<Token>,
    variables: HashMap<String, Variable>,
    tokens_on_line: Vec<Token>,
    line: usize,
    current_token: Token,
    index: usize,
    skip_next: bool,
    resolve_variables: bool,
    pub should_abort: bool,
}

impl Parser {
    pub fn parse(&mut self, tokens: LexedTokenLines) {
        #[cfg(debug_assertions)]
        println!(
            "--- Tokens ---\n{:?}\n--------------\n\n--- Output ---",
            tokens
        );

        for (line, mut tokens) in tokens.into_iter().enumerate() {
            self.tokens_on_line.clone_from(&tokens);
            tokens.reverse();

            self.line = line;

            for (i, token) in tokens.clone().into_iter().enumerate() {
                if self.should_abort {
                    #[cfg(debug_assertions)]
                    {
                        println!("--------------");

                        println!("--- Variables ---");
                        println!("{:?}", self.variables);
                        println!("-----------------");
                    }
                    return;
                }

                if self.skip_next {
                    self.skip_next = false;
                    continue;
                }

                self.current_token = token.clone();
                self.index = (tokens.len() - i) - 1;

                match token.token_type {
                    TokenType::Identifier => self.parse_identifier(token.value),
                    TokenType::Operator => self.parse_operator(token.value),
                    TokenType::Number | TokenType::String | TokenType::Boolean => {
                        self.stack.push(token)
                    }
                }
            }
        }

        #[cfg(debug_assertions)]
        {
            println!("--------------");

            println!("--- Variables ---");
            println!("{:?}", self.variables);
            println!("-----------------");
        }
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

            let to_print_if_number = if to_print == "0" {
                &format!("\x1b[33m{}\x1b[0m", to_print)
            } else {
                &format!("\x1b[33m{}\x1b[0m", to_print.trim_start_matches('0'))
            };

            let to_print_yellow = &format!("\x1b[33m{}\x1b[0m", to_print);

            to_print = match token.token_type {
                // Remove leading zeroes if the value is a number.
                TokenType::Number => to_print_if_number,
                TokenType::Boolean => to_print_yellow,
                TokenType::Identifier | TokenType::Operator | TokenType::String => to_print,
            };

            println!("{}", to_print);
        } else {
            let var = self.try_parse_variable(&identifier);
            if let Some(variable) = var {
                self.stack.push(variable);
            } else {
                self.error(&format!("Unknown identifier: `{}`", identifier));
            }
        }
    }

    fn parse_operator(&mut self, operator: String) {
        if operator == "+" {
            if let Some((a, b)) = self.get_surrounding_operator("+") {
                if a.token_type != TokenType::String && a.token_type != TokenType::Number {
                    self.error(&format!(
                        "`{}` expected a string or number, got identifier: `{}`",
                        operator, a.value
                    ));
                    return;
                } else if b.token_type != TokenType::String && b.token_type != TokenType::Number {
                    self.error(&format!(
                        "`{}` expected a string or number, got identifier: `{}`",
                        operator, b.value
                    ));
                    return;
                }

                let is_string =
                    a.token_type == TokenType::String || b.token_type == TokenType::String;

                if is_string {
                    self.stack.push(Token {
                        token_type: TokenType::String,
                        value: format!("{}{}", a.value, b.value),
                    });
                    return;
                }

                let a_number = a.value.parse::<i64>().unwrap_or_else(|_| {
                    self.error(&format!(
                        "Operator `+` expected a number to its left. Got `{}`",
                        a.value
                    ));

                    0
                });

                let b_number = b.value.parse::<i64>().unwrap_or_else(|_| {
                    self.error(&format!(
                        "Operator `+` expected a number to its right. Got `{}`",
                        b.value
                    ));

                    0
                });

                self.stack.push(Token {
                    token_type: TokenType::Number,
                    value: (a_number + b_number).to_string(),
                })
            }
        } else if operator == "=" {
            self.resolve_variables = false;

            if let Some((name, value)) = self.get_surrounding_operator("=") {
                if name.token_type != TokenType::Identifier {
                    self.error(&format!(
                        "`{}` expected an identifier on its left, got: `{:?}`",
                        operator, name.token_type
                    ));
                    return;
                } else if value.token_type != TokenType::String
                    && value.token_type != TokenType::Number
                    && value.token_type != TokenType::Boolean
                {
                    self.error(&format!(
                        "`{}` expected a string, number, or boolean, got identifier: `{}`",
                        operator, value.value
                    ));
                    return;
                }

                self.variables.insert(
                    name.value,
                    Variable {
                        variable_type: VariableType::from(value.token_type),
                        value: value.value,
                    },
                );
            }

            self.resolve_variables = true;
        } else if operator == "==" {
            if let Some((a, b)) = self.get_surrounding_operator("==") {
                if a.token_type != TokenType::String
                    && a.token_type != TokenType::Number
                    && a.token_type != TokenType::Boolean
                {
                    self.error(&format!(
                        "`{}` expected a string, number, or boolean, got identifier: `{}`",
                        operator, a.value
                    ));
                    return;
                } else if b.token_type != TokenType::String
                    && b.token_type != TokenType::Number
                    && b.token_type != TokenType::Boolean
                {
                    self.error(&format!(
                        "`{}` expected a string, number, or boolean, got identifier: `{}`",
                        operator, b.value
                    ));
                    return;
                }

                self.stack.push(Token {
                    token_type: TokenType::Boolean,
                    value: (a == b).to_string(),
                })
            }
        } else if operator == "!=" {
            if let Some((a, b)) = self.get_surrounding_operator("!=") {
                if a.token_type != TokenType::String
                    && a.token_type != TokenType::Number
                    && a.token_type != TokenType::Boolean
                {
                    self.error(&format!(
                        "`{}` expected a string, number, or boolean, got identifier: `{}`",
                        operator, a.value
                    ));
                    return;
                } else if b.token_type != TokenType::String
                    && b.token_type != TokenType::Number
                    && b.token_type != TokenType::Boolean
                {
                    self.error(&format!(
                        "`{}` expected a string, number, or boolean, got identifier: `{}`",
                        operator, b.value
                    ));
                    return;
                }

                self.stack.push(Token {
                    token_type: TokenType::Boolean,
                    value: (a != b).to_string(),
                })
            }
        } else if operator == ">" {
            if let Some((a, b)) = self.get_surrounding_operator(">") {
                if a.token_type != TokenType::Number {
                    self.error(&format!(
                        "`{}` expected a number, got identifier: `{}`",
                        operator, a.value
                    ));
                    return;
                } else if b.token_type != TokenType::Number {
                    self.error(&format!(
                        "`{}` expected a number, got identifier: `{}`",
                        operator, b.value
                    ));
                    return;
                }

                self.stack.push(Token {
                    token_type: TokenType::Boolean,
                    value: (a.value.parse::<i128>().unwrap() > b.value.parse::<i128>().unwrap())
                        .to_string(),
                })
            }
        } else if operator == "<" {
            if let Some((a, b)) = self.get_surrounding_operator("<") {
                if a.token_type != TokenType::Number {
                    self.error(&format!(
                        "`{}` expected a number, got identifier: `{}`",
                        operator, a.value
                    ));
                    return;
                } else if b.token_type != TokenType::Number {
                    self.error(&format!(
                        "`{}` expected a number, got identifier: `{}`",
                        operator, b.value
                    ));
                    return;
                }

                self.stack.push(Token {
                    token_type: TokenType::Boolean,
                    value: (a.value.parse::<i128>().unwrap() < b.value.parse::<i128>().unwrap())
                        .to_string(),
                })
            }
        } else if operator == ">=" {
            if let Some((a, b)) = self.get_surrounding_operator(">=") {
                if a.token_type != TokenType::Number {
                    self.error(&format!(
                        "`{}` expected a number, got identifier: `{}`",
                        operator, a.value
                    ));
                    return;
                } else if b.token_type != TokenType::Number {
                    self.error(&format!(
                        "`{}` expected a number, got identifier: `{}`",
                        operator, b.value
                    ));
                    return;
                }

                self.stack.push(Token {
                    token_type: TokenType::Boolean,
                    value: (a.value.parse::<i128>().unwrap() >= b.value.parse::<i128>().unwrap())
                        .to_string(),
                })
            }
        } else if operator == "<=" {
            if let Some((a, b)) = self.get_surrounding_operator("<=") {
                if a.token_type != TokenType::Number {
                    self.error(&format!(
                        "`{}` expected a number, got identifier: `{}`",
                        operator, a.value
                    ));
                    return;
                } else if b.token_type != TokenType::Number {
                    self.error(&format!(
                        "`{}` expected a number, got identifier: `{}`",
                        operator, b.value
                    ));
                    return;
                }

                self.stack.push(Token {
                    token_type: TokenType::Boolean,
                    value: (a.value.parse::<i128>().unwrap() <= b.value.parse::<i128>().unwrap())
                        .to_string(),
                })
            }
        } else if operator == "&&" {
            if let Some((a, b)) = self.get_surrounding_operator("&&") {
                if a.token_type != TokenType::Boolean {
                    self.error(&format!(
                        "`{}` expected a boolean, got identifier: `{}`",
                        operator, a.value
                    ));
                    return;
                } else if b.token_type != TokenType::Boolean {
                    self.error(&format!(
                        "`{}` expected a boolean, got identifier: `{}`",
                        operator, b.value
                    ));
                    return;
                }

                self.stack.push(Token {
                    token_type: TokenType::Boolean,
                    value: (a.value.parse().unwrap() && b.value.parse().unwrap()).to_string(),
                })
            }
        } else if operator == "||" {
            if let Some((a, b)) = self.get_surrounding_operator("||") {
                if a.token_type != TokenType::Boolean {
                    self.error(&format!(
                        "`{}` expected a boolean, got identifier: `{}`",
                        operator, a.value
                    ));
                    return;
                } else if b.token_type != TokenType::Boolean {
                    self.error(&format!(
                        "`{}` expected a boolean, got identifier: `{}`",
                        operator, b.value
                    ));
                    return;
                }

                self.stack.push(Token {
                    token_type: TokenType::Boolean,
                    value: (a.value.parse().unwrap() || b.value.parse().unwrap()).to_string(),
                })
            }
        } else {
            self.error(&format!("Unknown operator: `{}`", operator));
        }
    }

    #[allow(dead_code)]
    fn next_token(
        &mut self,
        identifier: &str,
        expected_argument_amount: u8,
        default: Option<Token>,
    ) -> Token {
        if self.tokens_on_line.len() <= self.index + 1 {
            return default.unwrap_or_else(|| {
                self.error(&format!(
                    "`{}` needs {} argument(s).",
                    identifier, expected_argument_amount
                ));

                Token::default()
            });
        }

        let to_return = self.tokens_on_line[self.index + 1].clone();

        let var = self.try_parse_variable(&to_return.value);
        if let Some(variable) = var {
            return variable;
        }

        to_return
    }

    #[allow(dead_code)]
    fn previous_token(
        &mut self,
        identifier: &str,
        expected_argument_amount: u8,
        default: Option<Token>,
    ) -> Token {
        if self.index == 0 {
            return default.unwrap_or_else(|| {
                self.error(&format!(
                    "`{}` needs {} argument(s).",
                    identifier, expected_argument_amount
                ));

                Token::default()
            });
        }

        let to_return = self.tokens_on_line[self.index - 1].clone();

        let var = self.try_parse_variable(&to_return.value);
        if let Some(variable) = var {
            return variable;
        }

        to_return
    }

    #[allow(dead_code)]
    fn try_next_token(&self) -> Option<Token> {
        if self.tokens_on_line.len() <= self.index + 1 {
            return None;
        }

        let to_return = self.tokens_on_line[self.index + 1].clone();

        let var = self.try_parse_variable(&to_return.value);
        if let Some(variable) = var {
            return Some(variable);
        }

        Some(to_return)
    }

    fn try_previous_token(&self) -> Option<Token> {
        if self.index == 0 {
            return None;
        }

        let to_return = self.tokens_on_line[self.index - 1].clone();

        let var = self.try_parse_variable(&to_return.value);
        if let Some(variable) = var {
            return Some(variable);
        }

        Some(to_return)
    }

    fn get_surrounding_operator(&mut self, operator: &str) -> Option<(Token, Token)> {
        let a = self.try_previous_token();
        let b = self.stack.pop();

        if a.is_none() {
            self.error(&format!(
                "Operator `{}` expected a token on its left, got nothing",
                operator
            ));
            return None;
        } else if b.is_none() {
            self.error(&format!(
                "Operator `{}` expected a token on its right, got nothing",
                operator
            ));
            return None;
        }

        let a = a.unwrap();
        let b = b.unwrap();

        self.skip_next = true;
        Some((a, b))
    }

    fn try_parse_variable(&self, identifier: &String) -> Option<Token> {
        if !self.resolve_variables {
            return None
        }

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
            self.error(&format!(
                "`{}` needs {} argument(s)",
                identifier, expected_argument_amount
            ));

            Token::default()
        })
    }

    fn error(&mut self, message: &str) {
        io::stderr()
            .write_all(
                format!(
                    "\x1b[31mERROR: {}. Error occurred on line {}.\x1b[0m\n",
                    message,
                    self.line + 1
                )
                .as_bytes(),
            )
            .expect("Encountered error while printing error. Error-ception!");

        self.should_abort = true;
    }
}

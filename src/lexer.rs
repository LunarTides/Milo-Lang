use crate::parser::VariableType;

pub type LexedTokenLines = Vec<Vec<Token>>;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TokenType {
    #[default]
    Identifier,
    Number,
    String,
    Boolean,
    Operator,
}

impl From<TokenType> for VariableType {
    fn from(token_type: TokenType) -> VariableType {
        match token_type {
            TokenType::String => VariableType::String,
            TokenType::Number => VariableType::Number,
            TokenType::Boolean => VariableType::Boolean,
            TokenType::Identifier | TokenType::Operator => panic!("Invalid type conversion."),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
}

#[derive(Default)]
pub struct Lexer {
    token: Token,
    token_index: usize,
    local_tokens: Vec<Token>,
    is_in_string: bool,
    is_in_number: bool,
}

impl Lexer {
    pub fn lex_code(&mut self, code: String) -> LexedTokenLines {
        #[cfg(debug_assertions)]
        println!("--- Code ---\n{}\n------------\n", code);

        let mut global_tokens: LexedTokenLines = vec![];
        let lines = code.split(|c| c == '\n' || c == ';').collect::<Vec<&str>>();

        for (line_index, line) in lines.clone().into_iter().enumerate() {
            if line.is_empty() {
                continue;
            }

            self.token_index = 0;

            // Filter away characters that you cant even have in strings.
            let chars: Vec<char> = line.chars().filter(|c| *c != '\r').collect();

            for (i, char) in chars.clone().into_iter().enumerate() {
                // Strings.
                if char == '"' {
                    if !self.is_in_string {
                        self.token.token_type = TokenType::String;
                        self.is_in_string = true;
                    } else {
                        self.push_token();
                    }

                    continue;
                }

                if self.is_in_string {
                    self.token.value += char.to_string().as_str();
                    continue;
                }

                // Comment logic.
                if char == '#' || (char == '/' && (i < chars.len() - 1 && chars[i + 1] == '/')) {
                    break;
                }

                // Opening brackets.
                if char == '(' || char == ' ' {
                    self.push_token();
                    continue;
                }

                // Numbers.
                if !(self.is_in_number
                    || self.is_in_string
                    || !char.is_numeric()
                    || i > 0 && chars[i - 1].is_alphanumeric())
                {
                    self.push_token();

                    self.token.token_type = TokenType::Number;
                    self.is_in_number = true
                } else if self.is_in_number && !char.is_numeric() {
                    self.push_token();
                }

                if char == '-' && i < chars.len() - 1 && chars[i + 1].is_numeric() {
                    self.push_token();

                    self.token.token_type = TokenType::Number;
                    self.is_in_number = true;

                    self.token.value += char.to_string().as_str();
                    continue;
                }

                if char == '='
                    || char == '+'
                    || char == '-'
                    || char == '*'
                    || char == '/'
                    || char == '^'
                    || char == '!'
                    || char == '>'
                    || char == '<'
                {
                    self.token.token_type = TokenType::Operator;

                    if i >= chars.len() || chars[i + 1] != '=' {
                        self.token.value += char.to_string().as_str();
                        self.push_token();
                        continue;
                    }
                }

                let current_token = self.current_token(&lines, line_index);

                if current_token == "false" || current_token == "true" {
                    self.token.token_type = TokenType::Boolean;
                }

                if current_token == "&&" || current_token == "||" {
                    self.token.token_type = TokenType::Operator;
                }

                // Ignore outside of strings.
                if char == ')' || char == ',' {
                    continue;
                }

                self.token.value += char.to_string().as_str();
            }

            self.push_token();
            global_tokens.push(self.local_tokens.clone());
            self.local_tokens.clear();
        }

        global_tokens
    }

    fn current_token(&self, lines: &[&str], line_index: usize) -> String {
        let tokens = lines[line_index]
            .split(|c: char| c == ' ' || c == '(')
            .collect::<Vec<&str>>();

        if self.token_index >= tokens.len() || lines[line_index].is_empty() {
            return String::new();
        }

        tokens[self.token_index].trim().to_string()
    }

    fn push_token(&mut self) {
        if self.token.value.is_empty() {
            return;
        }

        self.local_tokens.push(self.token.clone());
        self.token = Token::default();
        self.is_in_number = false;
        self.is_in_string = false;
        self.token_index += 1;
    }
}

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
    Symbol,
}

impl From<TokenType> for VariableType {
    fn from(token_type: TokenType) -> VariableType {
        match token_type {
            TokenType::String => VariableType::String,
            TokenType::Number => VariableType::Number,
            TokenType::Boolean => VariableType::Boolean,
            TokenType::Identifier | TokenType::Operator | TokenType::Symbol => {
                panic!("Invalid type conversion.")
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
}

const SYMBOLS: &[char] = &['(', ')', '[', ']', '{', '}'];
const OPERATORS: &[char] = &['=', '+', '-', '*', '/', '^', '!', '>', '<'];

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
        let code = code.replace("\r", "");

        #[cfg(debug_assertions)]
        println!("--- Code ---\n{}\n------------\n", code);

        let mut global_tokens: LexedTokenLines = vec![];
        let lines = code.split(['\n', ';']).collect::<Vec<&str>>();

        let mut seperators = vec![' '];
        seperators.extend_from_slice(SYMBOLS);
        seperators.extend_from_slice(OPERATORS);

        for (line_index, line) in lines.clone().into_iter().enumerate() {
            if line.is_empty() {
                continue;
            }

            self.token_index = 0;

            // Filter away characters that you cant even have in strings.
            let chars: Vec<char> = line.chars().collect();
            let mut last_token: String = String::new();

            for (i, _) in chars.clone().into_iter().enumerate() {
                let current_token = self.current_token(&lines, line_index);
                let char = current_token.chars().next();

                if char.is_none() {
                    break;
                }

                let char = char.unwrap();

                if current_token == last_token {
                    continue;
                }

                last_token = current_token.clone();

                // Ignore outside of strings.
                if self.token.value.contains(current_token.as_str()) {
                    continue;
                }

                // Strings.
                if char == '"' {
                    if !self.is_in_string {
                        self.token.token_type = TokenType::String;

                        self.token.value +=
                            current_token.replace('"', "").split("//").next().unwrap();

                        self.push_token();

                        if current_token.contains("//") || current_token.contains("#") {
                            break;
                        }
                    }

                    self.is_in_string = !self.is_in_string;
                    continue;
                }

                if self.is_in_string || self.is_in_number {
                    continue;
                }

                // Numbers.
                if char.is_numeric() {
                    if !self.is_in_number {
                        self.token.token_type = TokenType::Number;
                        self.token.value += current_token
                            .split(|c: char| !c.is_numeric())
                            .next()
                            .unwrap();

                        self.push_token();
                    }

                    self.is_in_number = !self.is_in_number;
                    continue;
                }

                // Comment logic.
                if char == '#'
                    || (char == '/' && (i < chars.len() - 1 && chars[i + 1] == '/'))
                    || current_token == "//"
                {
                    break;
                }

                if SYMBOLS.contains(&char) {
                    self.push_token();
                    self.token.token_type = TokenType::Symbol;
                    self.token.value = char.to_string();
                    self.push_token();
                    continue;
                }

                // Opening brackets.
                if char == ' ' {
                    self.push_token();
                    continue;
                }

                if char == '-' && i < chars.len() - 1 && chars[i + 1].is_numeric() {
                    self.push_token();

                    self.token.token_type = TokenType::Number;
                    self.is_in_number = true;

                    self.token.value += char.to_string().as_str();
                    continue;
                }

                if OPERATORS.contains(&char) {
                    self.token.token_type = TokenType::Operator;

                    if i >= chars.len() || chars[i + 1] != '=' {
                        self.token.value += char.to_string().as_str();
                        self.push_token();
                        continue;
                    }
                }

                if current_token == "false" || current_token == "true" {
                    self.token.token_type = TokenType::Boolean;
                }

                if current_token == "&&" || current_token == "||" {
                    self.token.token_type = TokenType::Operator;
                }

                self.token.value += current_token.as_str();
                self.push_token();
            }

            self.push_token();

            if !self.local_tokens.is_empty() {
                global_tokens.push(self.local_tokens.clone());
                self.local_tokens.clear();
            }
        }

        global_tokens
    }

    fn current_token(&mut self, lines: &[&str], line_index: usize) -> String {
        // let mut split = vec![' '];
        let mut split = vec![];
        split.extend_from_slice(SYMBOLS);
        // split.extend_from_slice(OPERATORS);

        let tokens = lines[line_index]
            .split(split.as_slice())
            .collect::<Vec<&str>>();

        if self.token_index >= tokens.len() || lines[line_index].is_empty() {
            return String::new();
        }

        tokens[self.token_index].trim().to_string()
    }

    fn push_token(&mut self) {
        self.is_in_number = false;
        self.is_in_string = false;

        if self.token.value.is_empty() {
            self.token = Token::default();
            return;
        }

        self.local_tokens.push(self.token.clone());
        self.token = Token::default();
        self.token_index += 1;
    }
}

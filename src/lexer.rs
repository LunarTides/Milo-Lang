use crate::parser::VariableType;

pub type LexedTokenLines = Vec<Vec<Token>>;

const SYMBOLS: &[char] = &['(', ')', '[', ']', '{', '}'];
const OPERATORS: &[char] = &['=', '+', '-', '*', '/', '^', '!', '>', '<', '&', '|'];

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

struct TokenizedLine {
    line_number: usize,
    tokens: Vec<String>,
}

impl Default for TokenizedLine {
    fn default() -> Self {
        TokenizedLine {
            // The default line number is MAX since if it starts at 0 the lexer will skip the first line.
            line_number: usize::MAX,
            tokens: Vec::default(),
        }
    }
}

#[derive(Default)]
pub struct Lexer {
    token: Token,
    token_index: usize,
    tokenized_line: TokenizedLine,
    local_tokens: Vec<Token>,
}

impl Lexer {
    pub fn lex_code(&mut self, code: String) -> LexedTokenLines {
        let code = code.replace("\r", "");

        #[cfg(debug_assertions)]
        println!("--- Code ---\n{}\n------------\n", code);

        let mut global_tokens: LexedTokenLines = vec![];
        let lines = code.split(['\n', ';']).collect::<Vec<&str>>();

        for (line_index, line) in lines.clone().into_iter().enumerate() {
            if line.is_empty() || line.trim().starts_with("//") {
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
                    self.token.token_type = TokenType::String;

                    self.token.value += current_token.replace('"', "").split("//").next().unwrap();

                    self.push_token();

                    if current_token.contains("//") {
                        break;
                    }

                    continue;
                }

                // Numbers.
                if char.is_numeric() {
                    self.token.token_type = TokenType::Number;
                    self.token.value += current_token
                        .split(|c: char| !c.is_numeric())
                        .next()
                        .unwrap();

                    self.push_token();

                    continue;
                }

                // Comment logic.
                if current_token == "//"
                    || (char == '/' && i < chars.len() - 1 && chars[i + 1] == '/')
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
        let mut split = vec![];
        split.extend_from_slice(SYMBOLS);
        split.extend_from_slice(OPERATORS);

        if self.tokenized_line.line_number != line_index {
            self.tokenized_line.tokens = self.split_tokens(lines[line_index].to_string(), |c| {
                split.as_slice().contains(&c)
            });

            self.tokenized_line.line_number = line_index;
        }

        let tokens = &self.tokenized_line.tokens;

        if self.token_index >= tokens.len() || lines[line_index].is_empty() {
            return String::new();
        }

        tokens[self.token_index].trim().to_string()
    }

    fn push_token(&mut self) {
        if self.token.value.is_empty() {
            self.token = Token::default();
            return;
        }

        self.local_tokens.push(self.token.clone());
        self.token = Token::default();
        self.token_index += 1;
    }

    // https://stackoverflow.com/a/40296745
    fn split_tokens<F>(&self, text: String, predicate: F) -> Vec<String>
    where
        F: Fn(char) -> bool,
    {
        let mut result = Vec::new();
        let mut last = 0;
        let mut is_in_string = false;
        let mut string_builder = String::new();
        for (index, matched) in text.match_indices(predicate) {
            if last != index {
                if is_in_string {
                    string_builder += &text[last..index];
                }

                if is_in_string && text[last..index].ends_with('"') {
                    is_in_string = false;
                    result.push(string_builder);
                    string_builder = String::new();
                    last = index + matched.len() - 1;
                    continue;
                }

                if text[last..index].starts_with('"') {
                    is_in_string = true;
                    continue;
                }

                result.push(text[last..index].to_string());
            }
            result.push(matched.to_string());
            last = index + matched.len();
        }

        if last < text.len() {
            let mut end = text.len();

            if text.ends_with([')', ']', '}']) {
                end -= 1;
            }

            result.push(text[last..end].to_string());
        }

        result
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Token {
    #[default]
    Identifier,
    Number,
    String,
}

pub type LexedToken = (String, Token);
pub type LexedTokens = Vec<Vec<LexedToken>>;

#[derive(Default)]
pub struct Lexer {
    token_key: String,
    token_type: Token,
    local_tokens: Vec<LexedToken>,
    is_in_string: bool,
    is_in_number: bool,
}

impl Lexer {
    pub fn lex_code(&mut self, code: String) -> LexedTokens {
        println!("{}", code);

        let mut global_tokens: LexedTokens = vec![];

        for line in code.split(|c| c == '\n' || c == ';') {
            for char in line.chars() {
                // Characters that you cant even have in strings.
                if char == '\r' {
                    continue;
                }

                // Strings.
                if char == '"' {
                    if !self.is_in_string {
                        self.token_type = Token::String;
                    } else {
                        self.push_token();
                        self.token_type = Token::Identifier;
                    }

                    self.is_in_string = !self.is_in_string;
                    continue;
                }

                if self.is_in_string {
                    self.token_key += char.to_string().as_str();
                    continue;
                }

                // Opening brackets.
                if char == '(' {
                    self.push_token();
                    continue;
                }

                // Ignore outside of strings.
                if char == ')' || char == ' ' {
                    continue;
                }

                // Numbers.
                if !self.is_in_number && !self.is_in_string && char.is_numeric() {
                    self.push_token();

                    self.token_type = Token::Number;
                    self.is_in_number = true
                } else if self.is_in_number && !char.is_numeric() {
                    self.push_token();

                    self.token_type = Token::Identifier;
                    self.is_in_number = false
                }

                self.token_key += char.to_string().as_str();
            }

            self.push_token();
            global_tokens.push(self.local_tokens.clone());
            self.local_tokens.clear();
        }

        global_tokens
    }

    fn push_token(&mut self) {
        if self.token_key == "" {
            return
        }

        self.local_tokens.push((self.token_key.clone(), self.token_type));
        self.token_key = String::new();
        self.token_type = Token::default();
    }
}

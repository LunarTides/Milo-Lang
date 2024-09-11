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
        println!("--- Code ---\n{}\n------------\n", code);

        let mut global_tokens: LexedTokens = vec![];

        for line in code.split(|c| c == '\n' || c == ';') {
            if line.is_empty() {
                continue;
            }

            // Filter away characters that you cant even have in strings.
            let chars: Vec<char> = line.chars().filter(|c| *c != '\r').collect();

            for (i, char) in chars.clone().into_iter().enumerate() {
                // Strings.
                if char == '"' {
                    if !self.is_in_string {
                        self.token_type = Token::String;
                        self.is_in_string = true;
                    } else {
                        self.push_token();
                    }

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

                // Numbers.
                if !self.is_in_number
                    && !self.is_in_string
                    && char.is_numeric()
                    && !chars[i - 1].is_alphanumeric()
                {
                    self.push_token();

                    self.token_type = Token::Number;
                    self.is_in_number = true
                } else if self.is_in_number && !char.is_numeric() {
                    self.push_token();
                }

                // Ignore outside of strings.
                if char == ')' || char == ' ' || char == ',' {
                    continue;
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
        if self.token_key.is_empty() {
            return;
        }

        self.local_tokens
            .push((self.token_key.clone(), self.token_type));
        self.token_key = String::new();
        self.token_type = Token::default();
        self.is_in_number = false;
        self.is_in_string = false;
    }
}

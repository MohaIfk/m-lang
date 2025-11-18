use crate::tokens::{TokenType, Token, get_keywork_hash_map};

pub struct Tokenizer<'a> {
    tokens: Vec<Token>,
    current: usize,
    start: usize,
    line: usize,
    has_error: bool,
    source: &'a str,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            tokens: vec![],
            current: 0,
            start: 0,
            line: 1,
            has_error: false,
            source,
        }
    }

    pub fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    fn peek(&self, index: usize) -> Option<u8> {
        self.source.as_bytes().get(self.current + index).cloned()
    }

    pub fn generate_tokens(&mut self) -> Result<(), String> {
        while let Some(c) = self.peek(0) {
            self.start = self.current;

            match c {
                b' ' | b'\t' | b'\r' => {
                    self.current += 1;
                    continue;
                }
                b'\n' => {
                    self.line += 1;
                    self.current += 1;
                    continue
                }
                _ => {}
            }

            let a = self.get_token(c);
            if a.is_err() {
                self.has_error = true;
                return Err(a.err().unwrap());
            } else {
                self.tokens.push(a?);
            }
        }
        self.tokens.push(Token::new(TokenType::EOF, "".to_string()));
        Ok(())
    }

    fn get_token(&mut self, c: u8) -> Result<Token, String> {

        if c.is_ascii_digit() {
            return Ok(self.get_number());
        }

        if c.is_ascii_alphabetic() {
            return Ok(self.get_keyword_or_identifier())
        }

        self.current += 1;
        match c {
            b'(' => Ok(self.craft_token(TokenType::LeftParen)),
            b')' => Ok(self.craft_token(TokenType::RightParen)),
            b'{' => Ok(self.craft_token(TokenType::LeftBrace)),
            b'}' => Ok(self.craft_token(TokenType::RightBrace)),
            b'[' => Ok(self.craft_token(TokenType::LeftBracket)),
            b']' => Ok(self.craft_token(TokenType::RightBracket)),
            b',' => Ok(self.craft_token(TokenType::Comma)),
            b'.' => Ok(self.craft_token(TokenType::Dot)),
            b':' => Ok(self.craft_token(TokenType::Colon)),
            b';' => Ok(self.craft_token(TokenType::Semicolon)),
            b'+' => Ok(self.craft_token(TokenType::Plus)),
            b'-' => {
                match self.peek(0) {
                    Some(d) => {
                        match d {
                            b'>' => {
                                self.current += 1;
                                Ok(self.craft_token(TokenType::Arrow))
                            },
                            _ => Ok(self.craft_token(TokenType::Minus)),
                        }
                    }
                    None => Ok(self.craft_token(TokenType::Minus)),
                }
            },
            b'*' => Ok(self.craft_token(TokenType::Star)),
            b'/' => Ok(self.craft_token(TokenType::Slash)),

            b'!' => {
                match self.peek(0) {
                    Some(d) => {
                        match d {
                            b'=' => {
                                self.current += 1;
                                Ok(self.craft_token(TokenType::BangEqual))
                            }
                            _ => Ok(self.craft_token(TokenType::Bang))
                        }
                    }
                    None => Ok(self.craft_token(TokenType::Bang))
                }
            },
            b'=' => {
                match self.peek(0) {
                    Some(d) => {
                        match d {
                            b'=' => {
                                self.current += 1;
                                Ok(self.craft_token(TokenType::EqualEqual))
                            }
                            _ => Ok(self.craft_token(TokenType::Equal))
                        }
                    }
                    None => Ok(self.craft_token(TokenType::Equal))
                }
            },
            b'<' => {
                match self.peek(0) {
                    Some(d) => {
                        match d {
                            b'=' => {
                                self.current += 1;
                                Ok(self.craft_token(TokenType::LessEqual))
                            },
                            _ => Ok(self.craft_token(TokenType::Less)),
                        }
                    }
                    None => Ok(self.craft_token(TokenType::Less)),
                }
            },
            b'>' => {
                match self.peek(0) {
                    Some(d) => {
                        match d {
                            b'=' => {
                                self.current += 1;
                                Ok(self.craft_token(TokenType::GreaterEqual))
                            },
                            _ => Ok(self.craft_token(TokenType::Greater)),
                        }
                    }
                    None => Ok(self.craft_token(TokenType::Greater)),
                }
            },
            b'"' => self.get_string(),
            _ => {
                self.has_error = true;
                Err(format!("unexpected character {:?}", c as char))
            }
        }
    }

    fn craft_token(&mut self, token_type: TokenType) -> Token {
        Token::new(token_type, self.source[self.start..self.current].to_string())
    }

    fn get_number(&mut self) -> Token {
        while let Some(c) = self.peek(0) {
            if c >= b'0' && c <= b'9' {
                self.current += 1;
            } else {
                break
            }
        }

        if let Some(c) = self.peek(0) {
            if c == b'.' {
                self.current += 1;
            }
            while let Some(c) = self.peek(0) {
                if c >= b'0' && c <= b'9' {
                    self.current += 1;
                } else {
                    break
                }
            }
        }

        self.craft_token(TokenType::Number)
    }

    fn get_keyword_or_identifier(&mut self) -> Token {
        while let Some(c) = self.peek(0) {
            if c.is_ascii_alphanumeric() || c == b'_' {
                self.current += 1;
            } else {
                break
            }
        }
        let a = &self.source[self.start..self.current];

        let khm = get_keywork_hash_map();
        if let Some(k) = khm.get(a).cloned() {
            return self.craft_token(k);

        }
        self.craft_token(TokenType::Identifier)
    }

    fn get_string(&mut self) -> Result<Token, String> {
        while let Some(c) = self.peek(0) {
            if c != b'"' {
                self.current += 1;
            } else {
                break
            }
        }
        if let Some(c) = self.peek(0) {
            if c != b'"' {
                return Err(format!("String must end with '\"' found {}", c as char));
            }
        } else {
            return Err("String must end with '\"'".to_string());
        }
        self.current += 1;
        Ok(Token::new(TokenType::String, self.source[self.start+1..self.current-1].to_string()))
    }
}
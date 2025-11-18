use std::collections::HashMap;
use crate::tokens::{TokenType, Token, get_keywork_hash_map};

pub struct Tokenizer<'a> {
    tokens: Vec<Token>,
    current: usize,
    start: usize,
    line: usize,
    has_error: bool,
    source: &'a str,
    keyword_hash_map: HashMap<&'static str, TokenType>
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
            keyword_hash_map: get_keywork_hash_map()
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
                    continue;
                }
                b'/' => {
                    if let Some(d) = self.peek(1) {
                        if d == b'/' {
                            while let Some(d) = self.peek(0) {
                                self.current += 1;
                                if d == b'\n' {
                                    self.line += 1;
                                    break;
                                }
                            }
                            continue;
                        }
                    }
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

        if c.is_ascii_alphabetic() || c == b'_' {
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
            b'+' => {
                match self.peek(0) {
                    Some(d) => {
                        match d {
                            b'=' => {
                                self.current += 1;
                                Ok(self.craft_token(TokenType::PlusEqual))
                            },
                            _ => Ok(self.craft_token(TokenType::Plus)),
                        }
                    }
                    None => Ok(self.craft_token(TokenType::Plus)),
                }
            },
            b'-' => {
                match self.peek(0) {
                    Some(d) => {
                        match d {
                            b'>' => {
                                self.current += 1;
                                Ok(self.craft_token(TokenType::Arrow))
                            },
                            b'=' => {
                                self.current += 1;
                                Ok(self.craft_token(TokenType::MinusEqual))
                            },
                            _ => Ok(self.craft_token(TokenType::Minus)),
                        }
                    }
                    None => Ok(self.craft_token(TokenType::Minus)),
                }
            },
            b'*' => {
                match self.peek(0) {
                    Some(d) => {
                        match d {
                            b'=' => {
                                self.current += 1;
                                Ok(self.craft_token(TokenType::StarEqual))
                            },
                            _ => Ok(self.craft_token(TokenType::Star)),
                        }
                    }
                    None => Ok(self.craft_token(TokenType::Star)),
                }
            },
            b'/' => {
                match self.peek(0) {
                    Some(d) => {
                        match d {
                            b'=' => {
                                self.current += 1;
                                Ok(self.craft_token(TokenType::SlashEqual))
                            },
                            _ => Ok(self.craft_token(TokenType::Slash)),
                        }
                    }
                    None => Ok(self.craft_token(TokenType::Slash)),
                }
            },
            b'|' => {
                match self.peek(0) {
                    Some(d) => {
                        if d == b'|' {
                            self.current += 1;
                            return Ok(self.craft_token(TokenType::PipePipe));
                        }
                        Ok(self.craft_token(TokenType::Pipe))
                    }
                    None => Ok(self.craft_token(TokenType::Pipe)),
                }
            },
            b'&' => {
                match self.peek(0) {
                    Some(d) => {
                        if d == b'&' {
                            self.current += 1;
                            return Ok(self.craft_token(TokenType::AmpersandAmpersand));
                        }
                        Ok(self.craft_token(TokenType::Ampersand))
                    }
                    None => Ok(self.craft_token(TokenType::Ampersand)),
                }
            },
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
                            }
                            b'<' => {
                                self.current += 1;
                                Ok(self.craft_token(TokenType::LessLess))
                            }
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
                            }
                            b'>' => {
                                self.current += 1;
                                Ok(self.craft_token(TokenType::GreaterGreater))
                            }
                            _ => Ok(self.craft_token(TokenType::Greater)),
                        }
                    }
                    None => Ok(self.craft_token(TokenType::Greater)),
                }
            },
            b'%' => {
                match self.peek(0) {
                    Some(d) => {
                        match d {
                            b'=' => {
                                self.current += 1;
                                Ok(self.craft_token(TokenType::PercentEqual))
                            },
                            _ => Ok(self.craft_token(TokenType::Percent)),
                        }
                    }
                    None => Ok(self.craft_token(TokenType::Percent)),
                }
            },
            b'^' => Ok(self.craft_token(TokenType::Caret)),
            b'~' => Ok(self.craft_token(TokenType::Tilde)),
            b'"' => self.get_string(),
            _ => {
                self.has_error = true;
                Err(format!("unexpected character {:?} at line {}", c as char, self.line))
            }
        }
    }

    fn craft_token(&mut self, token_type: TokenType) -> Token {
        Token::new(token_type, self.source[self.start..self.current].to_string())
    }

    fn get_number(&mut self) -> Token {
        while let Some(c) = self.peek(0) {
            if c.is_ascii_digit() {
                self.current += 1;
            } else {
                break
            }
        }

        if let Some(c) = self.peek(0) {
            if let Some(d) = self.peek(1)  {
                if c == b'.' && d.is_ascii_digit() { // we won't eat the dot if there is no digit ahead
                    self.current += 1;
                }
            }
            while let Some(c) = self.peek(0) {
                if c.is_ascii_digit() {
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

        if let Some(k) = self.keyword_hash_map.get(a).cloned() {
            return self.craft_token(k);

        }
        self.craft_token(TokenType::Identifier)
    }

    fn get_string(&mut self) -> Result<Token, String> {
        while let Some(c) = self.peek(0) {
            if c != b'"' {
                self.current += 1;
                if c == b'\n' { self.line += 1; }
                if c == b'\\' {
                    if let Some(d) = self.peek(0) {
                        if d == b'"' || d == b'\\' {
                            self.current += 1;
                            continue
                        }
                    }
                }
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
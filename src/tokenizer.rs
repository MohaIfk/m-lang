use std::collections::HashMap;
use crate::tokens::{TokenType, Token, get_keywork_hash_map};
use crate::error::Span;

pub struct Tokenizer<'a> {
    tokens: Vec<Token>,
    current: usize,
    col: usize,
    start: usize,
    start_col: usize,
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
            col: 1,
            start: 0,
            start_col: 1,
            line: 1,
            has_error: false,
            source,
            keyword_hash_map: get_keywork_hash_map()
        }
    }

    pub fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    fn advance(&mut self) {
        self.current+=1;
        self.col+=1;
    }

    fn peek(&self, index: usize) -> Option<u8> {
        self.source.as_bytes().get(self.current + index).cloned()
    }

    fn matches(&mut self, expected: u8) -> Option<u8> {
        if let Some(c) = self.source.as_bytes().get(self.current).cloned() {
            if c == expected {
                self.advance();
                return Some(c);
            }
        }
        None
    }

    pub fn generate_tokens(&mut self) -> Result<(), String> {
        while let Some(c) = self.peek(0) {
            self.start = self.current;
            self.start_col = self.col;

            match c {
                b' ' | b'\t' | b'\r' => {
                    self.advance();
                    continue;
                }
                b'\n' => {
                    self.line += 1;
                    self.col = 1;
                    self.current += 1;
                    continue;
                }
                b'/' => {
                    if let Some(d) = self.peek(1) {
                        if d == b'/' {
                            while let Some(d) = self.peek(0) {
                                self.advance();
                                if d == b'\n' {
                                    self.line += 1;
                                    self.col = 1;
                                    break;
                                }
                            }
                            continue;
                        } else if d == b'*' {
                            while let Some(d) = self.peek(0) {
                                self.advance();
                                if d == b'\n' {
                                    self.line += 1;
                                    self.col = 1;
                                }
                                if d == b'*' {
                                    if let Some(_) = self.matches(b'/') {
                                        break;
                                    }
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
        self.tokens.push(Token::new(TokenType::EOF, "".to_string(), Span::new(self.current, self.current, self.line, self.col)));
        Ok(())
    }

    fn get_token(&mut self, c: u8) -> Result<Token, String> {
        if c.is_ascii_digit() {
            return Ok(self.get_number());
        }

        if c.is_ascii_alphabetic() || c == b'_' {
            return Ok(self.get_keyword_or_identifier())
        }

        self.advance(); // Consume the character we peeked in generate_tokens
        match c {
            b'(' => Ok(self.craft_token(TokenType::LeftParen)),
            b')' => Ok(self.craft_token(TokenType::RightParen)),
            b'{' => Ok(self.craft_token(TokenType::LeftBrace)),
            b'}' => Ok(self.craft_token(TokenType::RightBrace)),
            b'[' => Ok(self.craft_token(TokenType::LeftBracket)),
            b']' => Ok(self.craft_token(TokenType::RightBracket)),
            b'@' => Ok(self.craft_token(TokenType::At)),
            b',' => Ok(self.craft_token(TokenType::Comma)),
            b'.' => Ok(self.craft_token(TokenType::Dot)),
            b':' => Ok(self.craft_token(TokenType::Colon)),
            b';' => Ok(self.craft_token(TokenType::Semicolon)),
            b'+' => {
                if let Some(_) = self.matches(b'=') {
                    Ok(self.craft_token(TokenType::PlusEqual))
                } else {
                    Ok(self.craft_token(TokenType::Plus))
                }
            },
            b'-' => {
                if let Some(_) = self.matches(b'=') {
                    Ok(self.craft_token(TokenType::MinusEqual))
                } else if let Some(_) = self.matches(b'>') {
                    Ok(self.craft_token(TokenType::Arrow))
                } else {
                    Ok(self.craft_token(TokenType::Minus))
                }
            },
            b'*' => {
                if let Some(_) = self.matches(b'=') {
                    Ok(self.craft_token(TokenType::StarEqual))
                } else {
                    Ok(self.craft_token(TokenType::Star))
                }
            },
            b'/' => {
                if let Some(_) = self.matches(b'=') {
                    Ok(self.craft_token(TokenType::SlashEqual))
                } else {
                    Ok(self.craft_token(TokenType::Slash))
                }
            },
            b'|' => {
                if let Some(_) = self.matches(b'|') {
                    Ok(self.craft_token(TokenType::PipePipe))
                } else {
                    Ok(self.craft_token(TokenType::Pipe))
                }
            },
            b'&' => {
                if let Some(_) = self.matches(b'&') {
                    Ok(self.craft_token(TokenType::AmpersandAmpersand))
                } else {
                    Ok(self.craft_token(TokenType::Ampersand))
                }
            },
            b'!' => {
                if let Some(_) = self.matches(b'=') {
                    Ok(self.craft_token(TokenType::BangEqual))
                } else {
                    Ok(self.craft_token(TokenType::Bang))
                }
            },
            b'=' => {
                if let Some(_) = self.matches(b'=') {
                    Ok(self.craft_token(TokenType::EqualEqual))
                } else {
                    Ok(self.craft_token(TokenType::Equal))
                }
            },
            b'<' => {
                if let Some(_) = self.matches(b'=') {
                    Ok(self.craft_token(TokenType::LessEqual))
                } else if let Some(_) = self.matches(b'<') {
                    Ok(self.craft_token(TokenType::LessLess))
                } else {
                    Ok(self.craft_token(TokenType::Less))
                }
            },
            b'>' => {
                if let Some(_) = self.matches(b'=') {
                    Ok(self.craft_token(TokenType::GreaterEqual))
                } else if let Some(_) = self.matches(b'>') {
                    Ok(self.craft_token(TokenType::GreaterGreater))
                } else {
                    Ok(self.craft_token(TokenType::Greater))
                }
            },
            b'%' => {
                if let Some(_) = self.matches(b'=') {
                    Ok(self.craft_token(TokenType::PercentEqual))
                } else {
                    Ok(self.craft_token(TokenType::Percent))
                }
            },
            b'^' => Ok(self.craft_token(TokenType::Caret)),
            b'~' => Ok(self.craft_token(TokenType::Tilde)),
            b'"' => self.get_string(),
            b'\'' => self.get_char(),
            _ => {
                self.has_error = true;
                Err(format!("unexpected character {:?} at line {}", c as char, self.line))
            }
        }
    }

    fn craft_token(&mut self, token_type: TokenType) -> Token {
        Token::new(token_type, self.source[self.start..self.current].to_string(), Span::new(self.start, self.current, self.line, self.start_col))
    }

    fn get_number(&mut self) -> Token {
        // Handle Hex (0x) and Binary (0b)
        if self.source.as_bytes()[self.start] == b'0' {
            self.advance();
            if let Some(c) = self.peek(0) {
                if c == b'x' || c == b'X' {
                    self.advance();
                    while let Some(h) = self.peek(0) {
                        if h.is_ascii_hexdigit() || h == b'_' {
                            self.advance();
                        } else {
                            break
                        }
                    }
                    return self.craft_token(TokenType::Int);
                } else if c == b'b' || c == b'B' {
                    self.advance();
                    while let Some(b) = self.peek(0) {
                        if b == b'0' || b == b'1' || b == b'_' {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    return self.craft_token(TokenType::Int);
                }
            }
        }

        let mut is_float: bool = false;
        while let Some(c) = self.peek(0) {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break
            }
        }

        if let Some(c) = self.peek(0) {
            if c == b'.' {
                if let Some(next) = self.peek(1) {
                    if next.is_ascii_digit() {
                        is_float = true;
                        self.advance();

                        while let Some(d) = self.peek(0) {
                            if d.is_ascii_digit() || d == b'_' {
                                self.advance();
                            } else {
                                break;
                            }
                        }

                        // Exponent notation (e.g. 1.2e10)
                        if let Some(e) = self.peek(0) {
                            if e == b'e' || e == b'E' {
                                self.advance();
                                if let Some(sign) = self.peek(0) {
                                    if sign == b'+' || sign == b'-' {
                                        self.advance();
                                    }
                                }
                                while let Some(d) = self.peek(0) {
                                    if d.is_ascii_digit() || d == b'_' {
                                        self.advance();
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.craft_token(if is_float { TokenType::Float } else { TokenType::Int })
    }

    fn get_keyword_or_identifier(&mut self) -> Token {
        while let Some(c) = self.peek(0) {
            if c.is_ascii_alphanumeric() || c == b'_' {
                self.advance();
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
                self.advance();
                if c == b'\n' {
                    self.line += 1;
                    self.col = 1;
                }
                if c == b'\\' {
                    if let Some(d) = self.peek(0) {
                        if d == b'"' || d == b'\\' {
                            self.advance();
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
        self.advance();
        Ok(Token::new(TokenType::String, self.source[self.start+1..self.current-1].to_string(), Span::new(self.start, self.current, self.line, self.start_col)))
    }

    fn get_char(&mut self) -> Result<Token, String> {
        if let Some(c) = self.peek(0) {
            self.advance();
            if c == b'\\' {
                if self.peek(0).is_some() {
                    self.advance();
                } else {
                    return Err("Unexpected EOF".to_string())
                }
            } else if c == b'\'' {
                return Err("Empty char literal".to_string());
            }
        } else {
            return Err("Unexpected EOF".to_string())
        }

        if let Some(c) = self.peek(0) {
            if c != b'\'' {
                return Err(format!("Char literal must end with \"'\", found {:?}", c as char));
            }
            self.advance();
        } else {
            return Err("Unexpected EOF".to_string())
        }
        Ok(Token::new(TokenType::Char, self.source[self.start+1..self.current-1].to_string(), Span::new(self.start, self.current, self.line, self.start_col)))
    }
}
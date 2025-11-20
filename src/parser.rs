use crate::tokens::{Token, TokenType};
use crate::ast::{Ast};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize
}

#[derive(Debug)]
pub struct ParseError {
    message: String,
    line: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0
        }
    }

    fn peek(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.current + n)
    }

    fn peek_type(&self) -> Option<TokenType> {
        if let Some(t) = self.peek(0) {
            Some(t.token_type.clone())
        } else {
            None
        }
    }

    fn previous(&self) -> Option<&Token> {
        self.tokens.get(self.current - 1)
    }

    fn advance(&mut self) {
        self.current += 1;
    }

    fn consume(&mut self, t: TokenType, msg: String) -> Result<Token, ParseError> {
        if let Some(_t) = self.tokens.get(self.current) {
            if _t.token_type == t {
                self.current += 1;
                return Ok(_t.clone());
            }
            return Err(ParseError {
                message: msg,
                line: _t.line,
            })
        }
        Err(ParseError {
            message: msg,
            line: 0,
        })
    }

    fn matches(&mut self, t: TokenType) -> Option<&Token> {
        if let Some(_t) = self.tokens.get(self.current) {
            if _t.token_type == t {
                self.current += 1;
                return Some(_t);
            }
        }
        None
    }
    fn m_matches(&mut self, t: Vec<TokenType>) -> Option<&Token> {
        if let Some(_t) = self.tokens.get(self.current) {
            for tt in t {
                if _t.token_type == tt {
                    self.current += 1;
                    return Some(_t);
                }
            }
        }
        None
    }

    // Program Structure
    pub fn parse_program(&mut self) -> Result<Ast, ParseError> {
        let mut top_levels: Vec<Box<Ast>> = vec![];
        while let Some(t) = self.tokens.get(self.current) {
            let a = match t.token_type {
                TokenType::EOF => break,
                _ => self.parse_top_level(t.clone())
            }?;
            top_levels.push(Box::new(a));
        }
        self.consume(TokenType::EOF, "program didn't end with EOF".to_string())?;
        Ok(Ast::Program {
            top_levels,
        })
    }

    pub fn parse_top_level(&mut self, t: Token) -> Result<Ast, ParseError> {
        let mut attr: Ast = Ast::None;
        let mut decl: Ast = Ast::None;
        if t.token_type == TokenType::At {
            self.advance();
            let name = self.consume(TokenType::Identifier, "expected Identifier after '@'".to_string())?.literal;
            let mut args: Vec<Box<Ast>> = vec![];
            if let Some(_) = self.matches(TokenType::LeftParen) {
                args = self.parse_arg_list()?;
                self.consume(TokenType::RightParen, "expected ')'".to_string())?;
            }
            attr = Ast::Attributes {
                name,
                args
            }
        }
        match t.token_type {
            _ => {}
        }
        Ok(Ast::TopLevel {
            attributes: Box::new(attr),
            declaration: Box::new(decl)
        })
    }

    // Expressions
    fn parse_expression(&mut self) -> Result<Box<Ast>, ParseError> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Box<Ast>, ParseError> {
        let mut expr = self.parse_logical_and()?;
        while let Some(t) = self.matches(TokenType::PipePipe).cloned() {
            let rhs = self.parse_logical_and()?;
            expr = Box::new(Ast::Binary {
                lhs: expr,
                op: t,
                rhs
            })
        }
        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Box<Ast>, ParseError> {
        let mut expr = self.parse_equality()?;
        while let Some(t) = self.matches(TokenType::AmpersandAmpersand).cloned() {
            let rhs = self.parse_equality()?;
            expr = Box::new(Ast::Binary {
                lhs: expr,
                op: t,
                rhs
            })
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Box<Ast>, ParseError> {
        let mut expr = self.parse_comparison()?;
        while let Some(t) = self.m_matches(vec![TokenType::EqualEqual, TokenType::BangEqual]).cloned() {
            let rhs = self.parse_comparison()?;
            expr = Box::new(Ast::Binary {
                lhs: expr,
                op: t,
                rhs
            })
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Box<Ast>, ParseError> {
        todo!()
    }

    fn parse_bitwise_or(&mut self) -> Result<Box<Ast>, ParseError> {
        todo!()
    }

    fn parse_bitwise_xor(&mut self) -> Result<Box<Ast>, ParseError> {
        todo!()
    }

    fn parse_bitwise_and(&mut self) -> Result<Box<Ast>, ParseError> {
        todo!()
    }

    fn parse_shift(&mut self) -> Result<Box<Ast>, ParseError> {
        todo!()
    }

    fn parse_term(&mut self) -> Result<Box<Ast>, ParseError> {
        todo!()
    }

    fn parse_factor(&mut self) -> Result<Box<Ast>, ParseError> {
        todo!()
    }

    fn parse_cast(&mut self) -> Result<Box<Ast>, ParseError> {
        todo!()
    }

    fn parse_unary(&mut self) -> Result<Box<Ast>, ParseError> {
        todo!()
    }

    fn parse_postfix(&mut self) -> Result<Box<Ast>, ParseError> {
        todo!()
    }

    fn parse_postfix_op(&mut self) -> Result<Box<Ast>, ParseError> {
        todo!()
    }

    fn parse_primary(&mut self) -> Result<Box<Ast>, ParseError> {
        todo!()
    }

    fn parse_struct_init(&mut self) -> Result<Box<Ast>, ParseError> {
        todo!()
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Box<Ast>>, ParseError> {
        let mut args: Vec<Box<Ast>> = vec![];
        Ok(args)
    }
}
use std::cmp::PartialEq;
use std::ptr::null;
use crate::tokens::{Token, TokenType};
use crate::ast::{Ast, Type};

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
            tokens, current: 0
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

    pub fn parse_program(&mut self) -> Result<Ast, ParseError> {
        let mut declarations: Vec<Box<Ast>> = Vec::new();
        while let Some(t) = self.tokens.get(self.current) {
            let a = match t.token_type {
                TokenType::EOF => break,
                _ => self.parse_declaration(t.clone())
            };
            declarations.push(Box::new(a?));
        }
        self.consume(TokenType::EOF, "program didn't end with EOF".to_string())?;
        Ok(Ast::Program {
            declarations,
        })
    }
    fn parse_declaration(&mut self, t:Token) -> Result<Ast, ParseError> {
        match t.token_type {
            //TokenType::Fn => self.parse_fn_declaration(),
            TokenType::Struct => self.parse_struct_declaration(),
            TokenType::Enum => self.parse_enum_declaration(),
            TokenType::Import => self.parse_import_statement(),
            _ => Err(ParseError {
                message: String::from("Declaration start with import, fn, enum, struct."),
                line: t.line
            })
        }
    }
    // i do what i do in declaration inside the program
    //fn parse_fn_declaration(&mut self) -> Ast {}
    fn parse_struct_declaration(&mut self) -> Result<Ast, ParseError> {
        self.consume(TokenType::Struct, "This must be a bug in the parser".to_string())?;
        let name = self.consume(TokenType::Identifier, "A string must be provided after struct".to_string())?.literal;
        let mut field_declarations: Vec<Box<Ast>> = Vec::new();
        self.consume(TokenType::LeftBrace, "we need {".to_string())?;
        while let Some(t) = self.peek(0) {
            if t.token_type == TokenType::RightBrace {
                break;
            }
            field_declarations.push(Box::new(self.parse_field_declaration()?));
        }
        self.consume(TokenType::RightBrace, "we need }".to_string())?;
        Ok(Ast::StructDeclaration{
            name,
            field_declarations
        })
    }

    fn parse_field_declaration(&mut self) -> Result<Ast, ParseError> {
        let name = self.consume(TokenType::Identifier, "A string must be provided after struct".to_string())?.literal;
        self.consume(TokenType::Colon, "we need : after a field name".to_string())?;
        let _type = self.parse_type()?;
        self.consume(TokenType::Semicolon, "we need ; after a field type".to_string())?;
        Ok(Ast::FieldDeclaration {
            name,
            type_: Box::new(_type),
        })
    }

    fn parse_type(&mut self) -> Result<Ast, ParseError> {
        if let Some(t) = self.tokens.get(self.current) {
            self.current += 1;

            match t.token_type {
                TokenType::Option => {
                    self.consume(TokenType::Less, "we need < after option".to_string())?;
                    let type_ = self.parse_type()?;
                    self.consume(TokenType::Greater, "we need > after option".to_string())?;
                    Ok(Ast::OptionType { type_: Box::new(type_) })
                },
                TokenType::Star => {
                    let type_ = self.parse_type()?;
                    Ok(Ast::PointerType { type_: Box::new(type_) })
                }
                TokenType::Identifier => {
                    Ok(Ast::UserDefinedType { name: t.literal.clone() })
                },
                _ => self.parse_primitive_type(t.clone())
            }
        } else {
            Err(ParseError{message: String::from("Unexpected EOF"), line: 0})
        }
    }

    fn parse_primitive_type(&mut self, t:Token) -> Result<Ast, ParseError> {
         match t.token_type {
            TokenType::CHAR => Ok(Ast::PrimitiveType { type_: Type::CHAR }),
            TokenType::U8 => Ok(Ast::PrimitiveType { type_: Type::U8 }),
            TokenType::U16 => Ok(Ast::PrimitiveType { type_: Type::U16 }),
            TokenType::U32 => Ok(Ast::PrimitiveType { type_: Type::U32 }),
            TokenType::U64 => Ok(Ast::PrimitiveType { type_: Type::U64 }),
            TokenType::I8 => Ok(Ast::PrimitiveType { type_: Type::I8 }),
            TokenType::I16 => Ok(Ast::PrimitiveType { type_: Type::I16 }),
            TokenType::I32 => Ok(Ast::PrimitiveType { type_: Type::I32 }),
            TokenType::I64 => Ok(Ast::PrimitiveType { type_: Type::I64 }),
            TokenType::F32 => Ok(Ast::PrimitiveType { type_: Type::F32 }),
            TokenType::F64 => Ok(Ast::PrimitiveType { type_: Type::F64 }),
            TokenType::BOOL => Ok(Ast::PrimitiveType { type_: Type::BOOL }),
            _ => Err(ParseError{message: String::from("Primitive type is not a primitive type"), line: t.line})
        }
    }

    fn parse_enum_declaration(&mut self) -> Result<Ast, ParseError> {
        self.consume(TokenType::Enum, "This must be a bug in the parser".to_string())?;
        let name = self.consume(TokenType::Identifier, "A string must be provided after enum".to_string())?.literal;
        self.consume(TokenType::LeftBrace, "we need { after a enum name".to_string())?;
        let mut enum_cases: Vec<Box<Ast>> = Vec::new();
        while let Some(t) = self.peek(0) {
            if t.token_type == TokenType::RightBrace {
                break;
            }
            enum_cases.push(Box::new(self.parse_enum_case()?));
        }
        self.consume(TokenType::RightBrace, "we need }".to_string())?;
        Ok(Ast::EnumDeclaration {
            name,
            enum_cases
        })
    }

    fn parse_enum_case(&mut self) -> Result<Ast, ParseError> {
        let name = self.consume(TokenType::Identifier, "A string must be provided after enum".to_string())?.literal;
        let mut member_types: Vec<Box<Ast>> = Vec::new();
        if let Some(_) = self.matches(TokenType::LeftParen) {
            member_types.push(Box::new(self.parse_type()?));
            while let Some(_) = self.matches(TokenType::Comma) {
                member_types.push(Box::new(self.parse_type()?));
            }
            self.consume(TokenType::RightParen, "we need )".to_string())?;
        }
        self.consume(TokenType::Semicolon, "we need ;".to_string())?;
        Ok(Ast::EnumCase {
            name,
            types_: member_types
        })
    }

    fn parse_import_statement(&mut self) -> Result<Ast, ParseError> {
        self.consume(TokenType::Import, "This must be a bug in the parser".to_string())?;
        let import_str = self.consume(TokenType::String, "A string must be provided after import".to_string())?.literal;
        self.consume(TokenType::Semicolon, "import statement must end with Semicolon".to_string())?;
        Ok(Ast::ImportStatement {
            name: import_str,
        })
    }
}
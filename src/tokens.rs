use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum TokenType {
    // Single-Character Tokens
    LeftParen,      // (
    RightParen,     // )
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Comma,          // ,
    Dot,            // .
    Colon,          // :
    Semicolon,      // ;
    Plus,           // +
    Minus,          // -
    Star,           // * (Used for multiplication AND pointers)
    Slash,          // /

    Bang,           // !
    Equal,          // =
    EqualEqual,     // ==
    BangEqual,      // !=
    Less,           // <
    LessEqual,      // <=
    Greater,        // >
    GreaterEqual,   // >=
    Arrow,          // -> (Function return type)

    Identifier,     // my_var, User, Arena
    Number,         // 123, 3.14
    String,         // "hello world"

    Fn, Struct, Enum, Import,
    Let, Var,

    If, ElseIf, Else,
    Match, Return, Jump,

    Arena,          // The type for memory allocators
    Unsafe,         // The keyword for 'unsafe' blocks
    As,             // For type casting: 'foo as *u8'

    True, False,
    F32, F64, BOOL, CHAR, // Primitive types
    I8, I16, I32, I64,
    U8, U16, U32, U64,

    EOF // End of File
}

#[derive(Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
}

impl Token {
    pub fn new(token_type: TokenType, literal: String) -> Self {
        Token { token_type, literal }
    }
}

pub fn get_keywork_hash_map() -> HashMap<&'static str, TokenType> {
    HashMap::from([
        ("fn",       TokenType::Fn),
        ("struct",   TokenType::Struct),
        ("enum",     TokenType::Enum),
        ("import",   TokenType::Import),
        ("let",      TokenType::Let),      // Immutable variable
        ("var",      TokenType::Var),      // Mutable variable

        ("if",       TokenType::If),
        ("elif",     TokenType::ElseIf),
        ("else",     TokenType::Else),
        ("match",    TokenType::Match),
        ("return",   TokenType::Return),
        ("jmp",   TokenType::Jump),

        ("Arena",    TokenType::Arena),
        ("unsafe",   TokenType::Unsafe),
        ("as",       TokenType::As),

        ("true",     TokenType::True),
        ("false",    TokenType::False),
        ("i8",       TokenType::I8),
        ("i16",      TokenType::I16),
        ("i32",      TokenType::I32),
        ("i64",      TokenType::I64),
        ("u8",       TokenType::U8),
        ("u16",      TokenType::U16),
        ("u32",      TokenType::U32),
        ("u64",      TokenType::U64),
        ("f32",      TokenType::F32),
        ("f64",      TokenType::F64),
        ("bool",     TokenType::BOOL),
        ("char",     TokenType::CHAR),
    ])
}
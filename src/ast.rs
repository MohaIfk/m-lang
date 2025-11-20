use crate::tokens::Token;

#[derive(Debug)]
pub enum Type {
    F32, F64, BOOL, CHAR,
    I8, I16, I32, I64,
    U8, U16, U32, U64
}

#[derive(Debug)]
pub enum Ast {
    Program { top_levels: Vec<Box<Ast>> },
    TopLevel { attributes: Box<Ast>, declaration: Box<Ast> },
    Attributes { name: String, args: Vec<Box<Ast>> },
    Binary {lhs: Box<Ast>, op: Token, rhs: Box<Ast>},
    None,
}
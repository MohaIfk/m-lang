use crate::tokens::TokenType;
use std::fmt;
use std::fmt::{Formatter};


#[derive(Debug)]
pub enum Type {
    F32, F64, BOOL, CHAR,
    I8, I16, I32, I64,
    U8, U16, U32, U64
}

#[derive(Debug)]
pub enum Ast {
    LiteralNumber(String),
    LiteralBool(bool),
    LiteralString(String),
    Binary{
        lhs: Box<Ast>,
        op: TokenType,
        rhs: Box<Ast>
    },
    Unary{
        op: TokenType,
        rhs: Box<Ast>
    },
    Program{
        declarations: Vec<Box<Ast>>,
    },
    ImportStatement {
        name: String,
    },
    StructDeclaration {
        name: String,
        field_declarations: Vec<Box<Ast>>,
    },
    FieldDeclaration {
        name: String,
        type_: Box<Ast>
    },
    EnumDeclaration {
        name: String,
        enum_cases: Vec<Box<Ast>>
    },
    EnumCase {
        name: String,
        types_: Vec<Box<Ast>>
    },
    UserDefinedType {
        name: String
    },
    OptionType {
       type_: Box<Ast>
    },
    PointerType {
        type_: Box<Ast>
    },
    PrimitiveType {
        type_: Type
    }
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Ast::LiteralNumber(s) => write!(f, "{}", s),
            Ast::LiteralBool(b) => write!(f, "{}", b),
            Ast::LiteralString(s) => write!(f, "{:?}", s),

            Ast::Binary{lhs, op, rhs} => write!(f, "(Binary {:?} {} {})", op, lhs, rhs),
            Ast::Unary{op, rhs} => write!(f, "(Unary {:?} {})", op, rhs),

            Ast::ImportStatement{name} => write!(f, "(Import {})", name),
            Ast::Program{declarations} => {
                writeln!(f, "(Program")?;
                for decl in declarations.iter() {
                    writeln!(f, "\t{}", decl)?;
                }
                write!(f, ")")
            }

            Ast::StructDeclaration { name, field_declarations } => {
                writeln!(f, "(StructDeclaration name={}", name)?;
                for field in field_declarations.iter() {
                    write!(f, "\t{}", field.to_string().replace("\n", "\n\t"))?;
                }
                write!(f, ")")
            }
            Ast::FieldDeclaration { name, type_ } => {
                write!(f, "(Field name={} type={})", name, type_)
            }

            Ast::UserDefinedType { name } => write!(f, "(UserType {})", name),
            Ast::OptionType { type_ } => write!(f, "(Option <{}>)", type_),
            Ast::PointerType { type_ } => write!(f, "(Pointer {})", type_),
            Ast::PrimitiveType { type_ } => write!(f, "(Primitive {:?})", type_),
            _ => write!(f, "Unknown AST Node")
        }
    }
}
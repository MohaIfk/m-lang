use crate::ast::Type::F64;
use crate::error::Span;
use crate::symbols::ScopeId;

#[derive(Debug, Clone)]
pub struct Node<T> {
    pub kind: T,
    pub span: Span,
    pub ty: Option<Type>, // for semantic analyzer
}

impl<T> Node<T> {
    pub fn new(kind: T, span: Span) -> Self {
        Self { kind, span, ty: None }
    }
}

// Syntactic Types
#[derive(Debug, Clone)]
pub enum TypeSpec {
    F32, F64, BOOL, CHAR,
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    Void,
    Named(String),
    Pointer(Box<TypeSpec>),
    Array(Box<TypeSpec>, Box<ExprNode>),
    Fn { params: Vec<TypeSpec>, ret: Box<TypeSpec> },
}

// Semantic Types
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    F32, F64, BOOL, CHAR,
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    UntypedInt(i128),
    UntypedFloat(f64),
    Struct(String),
    Enum(String),
    Pointer(Box<Type>),
    Array(Box<Type>, u64),
    Fn { params: Vec<Type>, ret: Box<Type> },
    Void,
    Error, // for type checker to mark error
}

pub enum CastSafety {
    Safe,
    Lossy,
    SignMismatch,
    FloatToInt,
    IntToFloat,
    PointerCast,
    PointerToInt,
    IntToPointer,
    Forbidden,
}

impl Type {
    pub fn size_in_bytes(&self) -> usize {
        use Type::*;
        match self {
            U8 | I8 | BOOL | CHAR => 1,
            U16 | I16 => 2,
            U32 | I32 | F32 => 4,
            U64 | I64 | F64 => 8,
            Pointer(_) | Fn { .. } => 8,
            Array(ty, len) => ty.size_in_bytes() * (*len as usize),
            Void => 0,
            Struct(_) | Enum(_) => 0,
            UntypedInt(_) | UntypedFloat(_) => 0,
            Error => 0,
        }
    }

    pub fn min_value(&self) -> i128 {
        match self {
            Type::F32 => unreachable!(),
            Type::F64 => unreachable!(),
            Type::BOOL => unreachable!(),
            Type::CHAR => 0,
            Type::I8 => i8::MIN as i128,
            Type::I16 => i16::MIN as i128,
            Type::I32 => i32::MIN as i128,
            Type::I64 => i64::MIN as i128,
            Type::U8 => u8::MIN as i128,
            Type::U16 => u16::MIN as i128,
            Type::U32 => u32::MIN as i128,
            Type::U64 => u64::MIN as i128,
            Type::UntypedInt(_) => unreachable!(),
            Type::UntypedFloat(_) => unreachable!(),
            Type::Struct(_) => unreachable!(),
            Type::Enum(_) => unreachable!(),
            Type::Pointer(_) => usize::MIN as i128,
            Type::Array(_, _) => unreachable!(),
            Type::Fn { .. } => unreachable!(),
            Type::Void => unreachable!(),
            Type::Error => unreachable!(),
        }
    }

    pub fn max_value(&self) -> i128 {
        match self {
            Type::F32 => unreachable!(),
            Type::F64 => unreachable!(),
            Type::BOOL => unreachable!(),
            Type::CHAR => 0,
            Type::I8 => i8::MAX as i128,
            Type::I16 => i16::MAX as i128,
            Type::I32 => i32::MAX as i128,
            Type::I64 => i64::MAX as i128,
            Type::U8 => u8::MAX as i128,
            Type::U16 => u16::MAX as i128,
            Type::U32 => u32::MAX as i128,
            Type::U64 => u64::MAX as i128,
            Type::UntypedInt(_) => unreachable!(),
            Type::UntypedFloat(_) => unreachable!(),
            Type::Struct(_) => unreachable!(),
            Type::Enum(_) => unreachable!(),
            Type::Pointer(_) => usize::MAX as i128,
            Type::Array(_, _) => unreachable!(),
            Type::Fn { .. } => unreachable!(),
            Type::Void => unreachable!(),
            Type::Error => unreachable!(),
        }
    }

    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    pub fn is_integer(&self) -> bool {
        use Type::*;
        matches!(self, I8|I16|I32|I64|U8|U16|U32|U64)
    }

    pub fn is_signed_int(&self) -> bool {
        use Type::*;
        matches!(self, I8|I16|I32|I64)
    }

    pub fn is_unsigned_int(&self) -> bool {
        use Type::*;
        matches!(self, U8|U16|U32|U64)
    }

    pub fn is_float(&self) -> bool {
        use Type::*;
        matches!(self, F32|F64)
    }

    pub fn is_pointer(&self) -> bool {
        use Type::*;
        matches!(self, Pointer(_))
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Type::Void)
    }

    pub fn try_unify_literal(&self, target: &Type) -> bool {
        match (self, target) {
            (Type::UntypedInt(val), t) if t.is_integer() => {
                let min = t.min_value();
                let max = t.max_value();
                *val >= min && *val <= max
            },
            (Type::UntypedFloat(val), t) if t.is_float() => true,
            (Type::UntypedInt(0), t) if t.is_pointer() => true, // We allow 0 to be a null pointer
            _ => false
        }
    }

    pub fn can_assign_from(&self, src: &Type) -> bool {
        if self == src { return true; }

        match (self, src) {
            (Type::Pointer(inner), Type::Pointer(_)) if inner.is_void() => true,
            (Type::Pointer(_), Type::Pointer(_)) => false,
            (dest, src) if dest.is_numeric() && src.is_numeric() => {
                matches!(src.check_cast_safety(dest), CastSafety::Safe)
            }
            _ => false
        }
    }

    pub fn check_cast_safety(&self, target: &Type) -> CastSafety {
        if self == target { return CastSafety::Safe; }
        use Type::*;

        match (self, target) {
            (t1, t2) if t1.is_integer() && t2.is_integer() => {
                let src_size = t1.size_in_bytes();
                let dst_size = t2.size_in_bytes();
                let src_signed = t1.is_signed_int();
                let dst_signed = t2.is_signed_int();

                if src_signed != dst_signed {
                    return CastSafety::SignMismatch;
                }

                if dst_size < src_size {
                    CastSafety::Lossy
                } else {
                    CastSafety::Safe
                }
            },

            (t1, t2) if t1.is_float() && t2.is_float() => {
                if t1.size_in_bytes() > t2.size_in_bytes() {
                    CastSafety::Lossy
                } else {
                    CastSafety::Safe
                }
            },

            (t1, t2) if t1.is_float() && t2.is_integer() => CastSafety::FloatToInt,
            (t1, t2) if t1.is_integer() && t2.is_float() => CastSafety::IntToFloat,

            (Pointer(_), t2) if t2.is_integer() => {
                if t2.size_in_bytes() < 8 { CastSafety::Lossy } else { CastSafety::PointerToInt }
            },
            (t1, Pointer(_)) if t1.is_integer() => CastSafety::IntToPointer,

            (Pointer(_), Pointer(_)) => CastSafety::PointerCast,

            (Enum(_), t2) if t2.is_integer() => CastSafety::Safe,
            (t1, Enum(_)) if t1.is_integer() => CastSafety::SignMismatch,

            _ => CastSafety::Forbidden
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Neg, Not, Deref, AddressOf
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr,
    Eq, Neq, Lt, Le, Gt, Ge,
}

#[derive(Debug, Clone)]
pub enum SizeOfTarget {
    Type(TypeSpec),
    Expr(Box<ExprNode>),
}

pub type ExprNode = Node<Expr>;
pub type StmtNode = Node<Stmt>;

impl ExprNode {
    pub fn is_lvalue(&self) -> bool {
        matches!(self.kind,
        Expr::Identifier(_) | Expr::Index{..} | Expr::MemberAccess{..} |
        Expr::Unary{op: UnaryOp::Deref, ..})
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    LiteralInt(u64),
    LiteralFloat(f64),
    LiteralChar(u8),
    LiteralString(String),
    LiteralBool(bool),
    Null,

    Identifier(String),

    Binary { lhs: Box<ExprNode>, op: BinaryOp, rhs: Box<ExprNode> },
    Unary { op: UnaryOp, rhs: Box<ExprNode> },
    Cast { expr: Box<ExprNode>, target: TypeSpec },
    Call { callee: Box<ExprNode>, args: Vec<ExprNode> },
    MemberAccess { object: Box<ExprNode>, member: String, is_arrow: bool },
    Index { array: Box<ExprNode>, index: Box<ExprNode> },

    SizeOf { target: SizeOfTarget },

    StructInit { name: String, fields: Vec<(String, ExprNode)> }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    // {...}
    Block(Vec<StmtNode>, Option<ScopeId>),

    // let x: i32 = 5;
    VarDecl{ is_mutable: bool, name: String, ty: TypeSpec, init: Option<ExprNode> },

    // x = 5;
    Assign{ target: ExprNode, value: ExprNode },

    If { condition: ExprNode, then_branch: Box<StmtNode>, else_branch: Option<Box<StmtNode>> },

    While { condition: ExprNode, body: Box<StmtNode> },

    For { init: Option<Box<StmtNode>>, condition: Option<ExprNode>, update: Option<Box<StmtNode>>, body: Box<StmtNode>, scope_id: Option<ScopeId> },

    Return(Option<ExprNode>),
    Break,
    Continue,

    // function call where ret is ignored
    Expression(ExprNode),
}

#[derive(Debug)]
pub struct FnDecl {
    pub attributes: Vec<Attribute>,
    pub name: String,
    pub params: Vec<(String, TypeSpec)>,
    pub return_type: TypeSpec,
    pub body: Box<StmtNode>,
    pub span: Span,
    pub scope_id: Option<ScopeId>,
}

#[derive(Debug)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<ExprNode>,
}

#[derive(Debug)]
pub struct StructDecl {
    pub attributes: Vec<Attribute>,
    pub name: String,
    pub fields: Vec<(String, TypeSpec)>,
    pub span: Span,
}

#[derive(Debug)]
pub struct EnumDecl {
    pub attributes: Vec<Attribute>,
    pub name: String,
    pub variants: Vec<(String, Option<ExprNode>)>,
    pub span: Span,
}

#[derive(Debug)]
pub struct ExternDecl {
    pub attributes: Vec<Attribute>,
    pub abi: String,
    pub name: String,
    pub params: Vec<(String, TypeSpec)>,
    pub return_type: TypeSpec,
    pub is_varargs: bool, // TODO: add support for C (printf)
    pub span: Span,
}

#[derive(Debug)]
pub struct GlobalDecl {
    pub attributes: Vec<Attribute>,
    pub is_const: bool,
    pub name: String,
    pub ty: TypeSpec,
    pub init: Option<ExprNode>,
    pub span: Span,
}


pub type ItemNode = Node<Item>;

#[derive(Debug)]
pub enum Item {
    // fn main() -> i32 { ... }
    Function(FnDecl),

    Struct(StructDecl),

    Enum(EnumDecl),

    Extern(ExternDecl),

    Global(GlobalDecl),

    Import(String),
}

#[derive(Debug)]
pub struct Program {
    pub modules: Vec<ItemNode>,
    // pub span: Span,
}

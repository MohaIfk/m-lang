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
    Named(String),
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
    Forbidden,
}

impl Type {
    pub fn size_in_bytes(&self) -> u8 {
        use Type::*;
        match self {
            U8 | I8 | BOOL | CHAR => 1,
            U16 | I16 => 2,
            U32 | I32 | F32 => 4,
            U64 | I64 | F64 => 8,
            _ => 0,
        }
    }

    pub fn is_numeric(&self) -> bool {
        use Type::*;
        matches!(self, F32|F64|I8|I16|I32|I64|U8|U16|U32|U64)
    }

    pub fn can_cast_to(&self, target: &Type) -> bool {
        use Type::*;
        match (self, target) {
            (t1, t2) if t1.is_numeric() && t2.is_numeric() => true,
            (Pointer(_), U64) => true,
            (U64, Pointer(_)) => true,
            (Pointer(_), Pointer(_)) => true,
            _ => false
        }
    }

    pub fn check_cast_safety(&self, target: &Type) -> CastSafety {
        if self == target { return CastSafety::Safe; }

        if self.is_numeric() && target.is_numeric() {
            let src_size = self.size_in_bytes();
            let dest_size = target.size_in_bytes();

            let src_signed = matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64);
            let dest_signed = matches!(target, Type::I8 | Type::I16 | Type::I32 | Type::I64);

            if src_signed != dest_signed {
                return CastSafety::SignMismatch;
            }

            if dest_size < src_size {
                return CastSafety::Lossy;
            }

            return CastSafety::Safe;
        }

        CastSafety::Forbidden
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Type::Void)
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

#[derive(Debug, Clone)]
pub enum Expr {
    LiteralInt(u64),
    LiteralFloat(f64),
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

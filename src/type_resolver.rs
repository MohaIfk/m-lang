use crate::ast::{BinaryOp, Expr, ExprNode, Type, TypeSpec};
use crate::error::{CompilerError, Span};

pub struct TypeResolver {
}

impl TypeResolver {
    pub fn new() -> Self {
        Self {}
    }

    pub fn resolve_type(&self, spec: &TypeSpec) -> Result<Type, (String, Span)> {
        match spec {
            TypeSpec::F32 => Ok(Type::F32),
            TypeSpec::F64 => Ok(Type::F64),
            TypeSpec::BOOL => Ok(Type::BOOL),
            TypeSpec::CHAR => Ok(Type::CHAR),
            TypeSpec::I8 => Ok(Type::I8),
            TypeSpec::I16 => Ok(Type::I16),
            TypeSpec::I32 => Ok(Type::I32),
            TypeSpec::I64 => Ok(Type::I64),
            TypeSpec::U8 => Ok(Type::U8),
            TypeSpec::U16 => Ok(Type::U16),
            TypeSpec::U32 => Ok(Type::U32),
            TypeSpec::U64 => Ok(Type::U64),
            TypeSpec::Usize => Ok(Type::Usize),
            TypeSpec::Isize => Ok(Type::Isize),
            TypeSpec::Void => Ok(Type::Void),
            TypeSpec::Named(name) => Ok(Type::Struct(name.clone())),
            TypeSpec::Pointer(inner) => {
                Ok(Type::Pointer(Box::new(self.resolve_type(inner)?)))
            },
            TypeSpec::Array(inner, size_expr) => {
                let resolved_inner = self.resolve_type(inner)?;

                let size: Option<u64> = self.eval(size_expr);

                match size {
                    Some(n) => Ok(Type::Array(Box::new(resolved_inner), n)),
                    None => Err(("Array size must be constant".into(), size_expr.span))
                }
            },
            TypeSpec::Fn { params, ret } => {
                let mut resolved_params: Vec<Type> = vec![];
                for param in params {
                    resolved_params.push(self.resolve_type(&param)?);
                }
                let resolved_return = self.resolve_type(ret)?;
                Ok(Type::Fn { params: resolved_params, ret: Box::new(resolved_return) })
            }
        }
    }

    pub fn eval(&self, expr: &ExprNode) -> Option<u64> {
        match &expr.kind {
            Expr::LiteralInt(n) => Some(*n),
            Expr::Binary { lhs, op: BinaryOp::Add, rhs } => {
                Some(self.eval(lhs)? + self.eval(rhs)?)
            },
            Expr::Binary { lhs, op: BinaryOp::Sub, rhs } => {
                Some(self.eval(lhs)? - self.eval(rhs)?)
            },
            Expr::Binary { lhs, op: BinaryOp::Mul, rhs } => {
                Some(self.eval(lhs)? * self.eval(rhs)?)
            },
            Expr::Binary { lhs, op: BinaryOp::Div, rhs } => {
                Some(self.eval(lhs)? / self.eval(rhs)?)
            },
            Expr::Binary { lhs, op: BinaryOp::Mod, rhs } => {
                Some(self.eval(lhs)? % self.eval(rhs)?)
            },
            Expr::Binary { lhs, op: BinaryOp::BitAnd, rhs } => {
                Some(self.eval(lhs)? & self.eval(rhs)?)
            },
            Expr::Binary { lhs, op: BinaryOp::BitOr, rhs } => {
                Some(self.eval(lhs)? | self.eval(rhs)?)
            },
            Expr::Binary { lhs, op: BinaryOp::BitXor, rhs } => {
                Some(self.eval(lhs)? ^ self.eval(rhs)?)
            },
            Expr::Binary { lhs, op: BinaryOp::Shl, rhs } => {
                Some(self.eval(lhs)? << self.eval(rhs)?)
            },
            Expr::Binary { lhs, op: BinaryOp::Shr, rhs } => {
                Some(self.eval(lhs)? >> self.eval(rhs)?)
            },
            _ => None
        }
    }

    pub fn determine_backing_type(min: i64, max: i64) -> Type {
        if min >= 0 && max <= u8::MAX as i64 {
            return Type::U8;
        }
        if min >= i8::MIN as i64 && max <= i8::MAX as i64 {
            return Type::I8;
        }
        if min >= 0 && max <= u16::MAX as i64 {
            return Type::U16;
        }
        Type::I32
    }
}
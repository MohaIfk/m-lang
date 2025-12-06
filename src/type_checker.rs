use std::collections::HashMap;
use crate::ast::*;
use crate::error::{CompilerError, Span};
use crate::symbols::*;
use crate::type_registry::{TypeInfo, TypeRegistry};
use crate::type_resolver::TypeResolver;
use crate::visitor::ASTVisitor;

pub struct TypeChecker<'a> {
    symbol_table: SymbolTable,
    type_registry: TypeRegistry,
    current_scope_id: ScopeId,
    type_resolver: TypeResolver,
    current_fn_return_type: Option<Type>,
    pub errors: Vec<CompilerError<'a>>,
    file_source: &'a String,
}

impl<'a> TypeChecker<'a> {
    pub fn new(symbol_table: SymbolTable, type_registry: TypeRegistry, file_source: &'a String) -> Self {
        Self { symbol_table, type_registry, current_scope_id: 0, type_resolver: TypeResolver::new(), current_fn_return_type: None, errors: vec![], file_source }
    }

    pub fn check_program(&mut self, program: &mut Program) {
        self.visit_program(program);
    }

    fn creat_compiler_error(&mut self, message: String, span: Span) {
        self.errors.push(CompilerError::new(message, span, self.file_source))
    }

    fn creat_compiler_warning(&mut self, message: String, span: Span) {
        self.errors.push(CompilerError::warning(message, span, self.file_source))
    }

    fn check_assignment(&mut self, target: &Type, value: &Type, span: Span) {
        if target == &Type::Error || value == &Type::Error {
            self.creat_compiler_error("Unable to resolve the type".into(), span);
            return;
        }

        if let Type::UntypedInt(v) = value {
            if value.try_unify_literal(target) {
                return;
            } else {
                self.creat_compiler_error(
                    format!("Literal {} does not fit into type {:?}", v, target),
                    span
                );
                return;
            }
        }

        if let Type::UntypedFloat(v) = value {
            if value.try_unify_literal(target) {
                return;
            } else {
                self.creat_compiler_error(
                    format!("Float literal {} does not fit into type {:?}", v, target),
                    span
                );
            }
        }

        if !target.can_assign_from(value) {
            match value.check_cast_safety(target) {
                CastSafety::Safe => {
                    self.creat_compiler_warning(
                        format!("Implicit widening {:?} -> {:?}. This is safe, but explicit cast recommended.", value, target),
                        span,
                    );
                },
                CastSafety::Lossy | CastSafety::FloatToInt | CastSafety::IntToFloat => {
                    self.creat_compiler_error(
                        format!("Possible data loss! {:?} does not fit into {:?}. Use 'as' if this is intentional.", value, target),
                        span
                    );
                },
                CastSafety::SignMismatch => {
                    self.creat_compiler_error(
                        format!("Sign mismatch! {:?} -> {:?}. Assigning signed to unsigned (or vice versa) requires explicit cast.", value, target),
                        span
                    );
                },
                CastSafety::PointerCast | CastSafety::IntToPointer | CastSafety::PointerToInt => {
                    self.creat_compiler_error(
                        format!("Pointer type mismatch {:?} -> {:?}. Use explicit 'as' cast for pointer manipulation.", value, target),
                        span
                    );
                },
                CastSafety::Forbidden => {
                    self.creat_compiler_error(
                        format!("Type Mismatch: Cannot convert {:?} to {:?}", value, target),
                        span
                    );
                }
            }
        }
    }

    fn resolve_type(&mut self, ty: &TypeSpec) -> Type {
        match self.type_resolver.resolve_type(ty) {
            Ok(resolved_ty) => resolved_ty,
            Err((msg, span)) => {
                self.creat_compiler_error(msg, span);
                Type::Error
            }
        }
    }

    fn is_truthy(&self, ty: &Type) -> bool {
        ty.is_numeric() || matches!(ty, Type::BOOL | Type::Pointer(..))
    }
}

impl<'a> ASTVisitor<()> for TypeChecker<'a> {
    fn visit_program(&mut self, program: &mut Program) -> () {
        for item in &mut program.modules {
            self.visit_item(item);
        }
    }

    fn visit_item(&mut self, item: &mut ItemNode) -> () {
        match &mut item.kind {
            Item::Function(f) => self.visit_fn_decl(f),
            Item::Struct(s) => self.visit_struct_decl(s),
            Item::Enum(e) => self.visit_enum_decl(e),
            Item::Extern(e) => self.visit_extern_decl(e),
            Item::Global(g) => self.visit_global_decl(g),
            Item::Import(_) => {},
        }
    }

    fn visit_fn_decl(&mut self, decl: &mut FnDecl) -> () {
        self.current_fn_return_type = Some(self.resolve_type(&decl.return_type).clone());
        self.visit_stmt(&mut decl.body);
        self.current_fn_return_type = None;
    }

    fn visit_struct_decl(&mut self, decl: &mut StructDecl) -> () {

    }

    fn visit_enum_decl(&mut self, decl: &mut EnumDecl) -> () {

    }

    fn visit_extern_decl(&mut self, decl: &mut ExternDecl) -> () {

    }

    fn visit_global_decl(&mut self, decl: &mut GlobalDecl) -> () {
        if let Some(init) = &mut decl.init {
            let resolved_ty = self.resolve_type(&decl.ty);
            self.visit_expr(init);
            let init_ty = init.ty.as_ref().unwrap();
            if resolved_ty != *init_ty {
                self.creat_compiler_error(format!("Type Mismatch in declaration expected: {:?}, found: {:?}", resolved_ty, init_ty), init.span);
            }
        }
    }

    fn visit_stmt(&mut self, stmt: &mut StmtNode) -> () {
        match &mut stmt.kind {
            Stmt::Block(stmts, scope_id) => {
                let temp_scope_id = self.current_scope_id;
                self.current_scope_id = scope_id.unwrap();
                for s in stmts {
                    self.visit_stmt(s);
                }
                self.current_scope_id = temp_scope_id;
            },
            Stmt::VarDecl { name: _, ty: type_spec, init, .. } => {
                let resolved_ty = self.resolve_type(type_spec);

                if let Some(init_expr) = init {
                    self.visit_expr(init_expr);
                    let init_ty = init_expr.ty.as_ref().unwrap();

                    self.check_assignment(&resolved_ty, init_ty, init_expr.span);
                }
            },
            Stmt::Assign { target, value } => {
                self.visit_expr(target);
                let target_ty = target.ty.as_ref().unwrap();
                self.visit_expr(value);
                let value_ty = value.ty.as_ref().unwrap();
                if target_ty.can_assign_from(value_ty) {
                    return;
                }
                self.check_assignment(target_ty, value_ty, value.span);
            },
            Stmt::If { condition, then_branch, else_branch } => {
                self.visit_expr(condition);
                let condition_ty = condition.ty.as_ref().unwrap();
                if !self.is_truthy(condition_ty) {
                    self.creat_compiler_error(format!("Condition must be a boolean or numeric, found {:?}", condition_ty), condition.span);
                }
                self.visit_stmt(then_branch);
                if let Some(else_b) = else_branch {
                    self.visit_stmt(else_b);
                }
            },
            Stmt::While { condition, body } => {
                self.visit_expr(condition);
                let condition_ty = condition.ty.as_ref().unwrap();
                if !self.is_truthy(condition_ty) {
                    self.creat_compiler_error(format!("Condition must be a boolean or numeric, found {:?}", condition_ty), condition.span);
                }
                self.visit_stmt(body);
            },
            Stmt::For{ init, condition, update, body, scope_id } => {
                let temp_scope_id = self.current_scope_id;
                self.current_scope_id = scope_id.unwrap();
                if let Some(stmt_init) = init {
                    self.visit_stmt(stmt_init);
                }
                if let Some(expr_condition) = condition {
                    self.visit_expr(expr_condition);
                    let condition_ty = expr_condition.ty.as_ref().unwrap();
                    if !self.is_truthy(condition_ty) {
                        self.creat_compiler_error(format!("Condition must be a boolean or numeric, found {:?}", condition_ty), expr_condition.span);
                    }
                }
                if let Some(stmt_update) = update {
                    self.visit_stmt(stmt_update);
                }
                self.current_scope_id = temp_scope_id;
            }
            Stmt::Expression(e) => { self.visit_expr(e); },
            Stmt::Return(result) => {
                let return_ty;
                let target_ty = self.current_fn_return_type.clone().unwrap();
                if let Some(expr) = result {
                    self.visit_expr(expr);
                    return_ty = expr.ty.as_ref().unwrap();
                } else {
                    return_ty = &Type::Void;
                }
                self.check_assignment(&target_ty, return_ty, stmt.span);
            },
            Stmt::Break | Stmt::Continue => {}
        }
    }

    fn visit_expr(&mut self, expr: &mut ExprNode) -> () {
        match &mut expr.kind {
            Expr::Binary { lhs, op, rhs } => {
                self.visit_expr(lhs);
                self.visit_expr(rhs);
                let left_ty = lhs.ty.as_ref().unwrap();
                let right_ty = rhs.ty.as_ref().unwrap();
                match op {
                    BinaryOp::Add | BinaryOp::Sub => expr.ty = {
                        match (left_ty, right_ty) {
                            (concrete, Type::UntypedInt(val)) | (Type::UntypedInt(val), concrete)
                            if concrete.is_integer() => {
                                if Type::UntypedInt(*val).try_unify_literal(concrete) {
                                    Some(concrete.clone())
                                } else {
                                    self.creat_compiler_error(
                                        format!("Literal {} does not fit into type {:?}", val, concrete),
                                        expr.span
                                    );
                                    Some(Type::Error)
                                }
                            },
                            (Type::UntypedInt(lval), Type::UntypedInt(rval)) => {
                                let r = match op {
                                    BinaryOp::Add => lval + rval,
                                    BinaryOp::Sub => lval - rval,
                                    _ => unreachable!()
                                };
                                Some(Type::UntypedInt(r))
                            },
                            (t1, t2) if t1 == t2 && t1.is_numeric() => Some(t1.clone()),
                            (Type::Pointer(inner), t2) if t2.is_integer() => Some(Type::Pointer(inner.clone())),
                            (Type::Pointer(t1), Type::Pointer(t2)) if t1 == t2 && matches!(op, BinaryOp::Sub) => Some(Type::U64),
                            _ => {
                                self.creat_compiler_error(format!("Invalid binary op {:?} between {:?} and {:?}", op, left_ty, right_ty), expr.span);
                                Some(Type::Error)
                            }
                        }
                    },
                    BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => expr.ty = {
                        if left_ty == right_ty && left_ty.is_numeric() {
                            Some(left_ty.clone())
                        } else {
                            self.creat_compiler_error(format!("Type Mismatch in binary op between: {:?} and {:?}", left_ty, right_ty), expr.span);
                            Some(Type::Error)
                        }
                    },
                    BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor | BinaryOp::Shl | BinaryOp::Shr => expr.ty = {
                        match (left_ty, right_ty) {
                            (t1, t2) if t1.is_numeric() && t2.is_numeric() => Some(right_ty.clone()),
                            (t1, t2) if t1.size_in_bytes() != 0 && t1.size_in_bytes() == t2.size_in_bytes() => Some(right_ty.clone()),
                            (_, _) => {
                                self.creat_compiler_error(format!("Type Mismatch in binary op between: {:?} and {:?}", left_ty, right_ty), expr.span);
                                Some(Type::Error)
                            }
                        }
                    },
                    BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Le | BinaryOp::Ge | BinaryOp::Lt | BinaryOp::Gt => expr.ty = Some(Type::BOOL),
                    BinaryOp::And | BinaryOp::Or => expr.ty = Some(Type::BOOL),
                }
            },
            Expr::Unary {op, rhs} => expr.ty = {
                match op {
                    UnaryOp::Neg => {
                        self.visit_expr(rhs);
                        let rhs_ty = rhs.ty.as_ref().unwrap();
                        if rhs_ty.is_numeric() {
                            Some(rhs_ty.clone())
                        } else {
                            Some(Type::Error)
                        }
                    }
                    UnaryOp::Not => {
                        self.visit_expr(rhs);
                        let rhs_ty = rhs.ty.as_ref().unwrap();
                        if *rhs_ty == Type::BOOL {
                            Some(Type::BOOL)
                        } else {
                            Some(Type::Error)
                        }
                    }
                    UnaryOp::Deref => {
                        self.visit_expr(rhs);
                        let rhs_ty = rhs.ty.as_ref().unwrap();
                        if let Type::Pointer(t) = rhs_ty {
                            Some(*t.clone())
                        } else {
                            Some(Type::Error)
                        }
                    }
                    UnaryOp::AddressOf => {
                        self.visit_expr(rhs);
                        if !rhs.is_lvalue() {
                            self.creat_compiler_error("Cannot take address of temporary value".into(), expr.span);
                        }
                        let rhs_ty = rhs.ty.as_ref().unwrap();
                        Some(Type::Pointer(Box::new(rhs_ty.clone())))
                    }
                }
            },
            Expr::LiteralInt(n) => expr.ty = {
                Some(Type::UntypedInt(*n as i128))
            },
            Expr::LiteralFloat(_) => expr.ty = Some(Type::F32),
            Expr::LiteralChar(_) => expr.ty = Some(Type::CHAR),
            Expr::LiteralBool(_) => expr.ty = Some(Type::BOOL),
            Expr::LiteralString(_) => expr.ty = Some(Type::Pointer(Box::new(Type::CHAR))),
            Expr::Identifier(name) => {
                if let Some(ty) = self.symbol_table.resolve_type_from(name.clone(), self.current_scope_id) {
                    expr.ty = Some(self.resolve_type(&ty));
                } else {
                    expr.ty = Some(Type::Error);
                    self.creat_compiler_error(format!("Unresolved identifier: {}", name), expr.span);
                }
            },
            Expr::SizeOf {target} => {
                expr.ty = Some(Type::U64);
            },
            Expr::Cast{expr: _expr , target} => {
                self.visit_expr(_expr);
                expr.ty = Some(self.resolve_type(target));
            },
            Expr::Null => expr.ty = Some(Type::Pointer(Box::new(Type::Void))),
            Expr::Index{array, .. } => {
                self.visit_expr(array);
                let array_ty = array.ty.as_ref().unwrap();
                match array_ty {
                    Type::Pointer(target_ty) => expr.ty = Some(*target_ty.clone()),
                    Type::Array(elem_ty, _) => expr.ty = Some(*elem_ty.clone()),
                    _ => expr.ty = Some(Type::Error),
                }
            },
            Expr::MemberAccess {object, member, is_arrow} => {
                self.visit_expr(object);
                let object_ty = object.ty.clone().unwrap_or(Type::Error);
                let struct_name = if *is_arrow {
                    match object_ty {
                        Type::Pointer(inner_ty) => match *inner_ty {
                            Type::Struct(name) => name.clone(),
                            _ => {
                                self.creat_compiler_error(format!("Type '{:?}' is not a pointer to a struct, cannot use '->'", inner_ty), object.span);
                                expr.ty = Some(Type::Error);
                                return;
                            }
                        }
                        _ => {
                            self.creat_compiler_error("Arrow operator '->' used on non-pointer type".into(), object.span);
                            expr.ty = Some(Type::Error);
                            return;
                        }
                    }
                } else {
                    match object_ty {
                        Type::Struct(ref name) => name.clone(),
                        Type::Pointer(_) => {
                            self.creat_compiler_error(
                                "Cannot access members of a pointer to a struct, use '->' instead of '.'".into(),
                                object.span
                            );
                            expr.ty = Some(Type::Error);
                            return;
                        },
                        _ => {
                            self.creat_compiler_error(format!("Type '{:?}' is not a struct, cannot access members", object_ty), object.span);
                            expr.ty = Some(Type::Error);
                            return;
                        }
                    }
                };
                if let Some(struct_def) = self.type_registry.get_type(&struct_name) {
                    if let TypeInfo::Struct{fields, ..} = struct_def {
                        let field_ty = fields.iter()
                            .find(|(f_name, _)| f_name == member)
                            .map(|(_, f_ty)| f_ty.clone());
                        match field_ty {
                            Some(ty) => {
                                expr.ty = Some(ty);
                            },
                            None => {
                                self.creat_compiler_error(format!("Struct '{}' has no field named '{}'", struct_name, member), expr.span);
                                expr.ty = Some(Type::Error);
                            }
                        }
                    } else {
                        self.creat_compiler_error(format!("Type '{}' is not a struct", struct_name), expr.span);
                        expr.ty = Some(Type::Error);
                    }
                } else {
                    self.creat_compiler_error(format!("Type '{}' is not defined", struct_name), expr.span);
                    expr.ty = Some(Type::Error);
                }
            },
            Expr::Call {callee, args} => {
                self.visit_expr(callee);
                let callee_ty = callee.ty.as_ref().unwrap();
                if let Type::Fn {params, ret} = callee_ty {
                    if args.len() != params.len() {
                        self.creat_compiler_error(format!("Expected {} args, found {}", params.len(), args.len()), expr.span);
                    }

                    let limit = std::cmp::min(args.len(), params.len());

                    for i in 0..limit {
                        self.visit_expr(&mut args[i]);
                        let arg_ty = args[i].ty.as_ref().unwrap();
                        let param_ty = &params[i];
                        self.check_assignment(param_ty, arg_ty, args[i].span);
                    }

                    for i in limit..args.len() {
                        self.visit_expr(&mut args[i]);
                    }
                    expr.ty = Some(*ret.clone());
                } else {
                    expr.ty = Some(Type::Error);
                    self.creat_compiler_error(format!("Type {:?} is not callable", callee_ty), expr.span);
                }
            },
            Expr::StructInit {name: struct_name, fields } => {
                let mut struct_fields: HashMap<String, Type> = HashMap::new();
                if let Some(TypeInfo::Struct{fields: _fields, ..}) = self.type_registry.get_type(&struct_name) {
                    for (_f_name, _f_ty) in _fields {
                        struct_fields.insert(_f_name, _f_ty);
                    }
                }
                for (f_name, f_expr) in fields {
                    self.visit_expr(f_expr);
                    let f_expr_ty = f_expr.ty.as_ref().unwrap();
                    match struct_fields.get(f_name) {
                        Some(f_target_type) => {
                            if !f_target_type.can_assign_from(f_expr_ty) {
                                self.creat_compiler_error(
                                    format!("Type Mismatch: Cannot convert {:?} to {:?}", f_expr_ty, f_target_type),
                                    f_expr.span
                                );
                            }
                        },
                        None => {
                            self.creat_compiler_error(format!("Struct '{}' has no field named '{}'", struct_name, f_name), f_expr.span);
                            f_expr.ty = Some(Type::Error);
                        }
                    }
                }
                expr.ty = Some(Type::Struct(struct_name.clone()));
            },
        }
    }
}

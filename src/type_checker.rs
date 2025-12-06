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

    fn resolve_common_type(&mut self, lhs: &mut ExprNode, rhs: &mut ExprNode, op: BinaryOp) -> Option<Type> {
        let l_ty = lhs.ty.as_ref().unwrap().clone();
        let r_ty = rhs.ty.as_ref().unwrap().clone();

        match (l_ty, r_ty) {
            (Type::UntypedInt(l), Type::UntypedInt(r)) => {
                let res = match op {
                    BinaryOp::Add => l + r,
                    BinaryOp::Sub => l - r,
                    BinaryOp::Mul => l * r,
                    BinaryOp::Div => if r != 0 { l / r } else { self.creat_compiler_error("div by 0".into(), rhs.span); 0 },
                    BinaryOp::Mod => if r != 0 { l % r } else { self.creat_compiler_error("div by 0".into(), rhs.span); 0 },
                    BinaryOp::BitAnd => l & r,
                    BinaryOp::BitOr => l | r,
                    BinaryOp::BitXor => l ^ r,
                    BinaryOp::Shl => l << r,
                    BinaryOp::Shr => l >> r,
                    _ => 0
                };
                Some(Type::UntypedInt(res))
            },
            (Type::UntypedFloat(l), Type::UntypedFloat(r)) => {
                let res = match op {
                    BinaryOp::Add => l + r,
                    BinaryOp::Sub => l - r,
                    BinaryOp::Mul => l * r,
                    BinaryOp::Div => if r != 0.0 { l / r } else { self.creat_compiler_error("div by 0".into(), rhs.span); 0.0 },
                    BinaryOp::Mod => if r != 0.0 { l % r } else { self.creat_compiler_error("div by 0".into(), rhs.span); 0.0 },
                    _ => 0.0,
                };
                Some(Type::UntypedFloat(res))
            },

            (t1, t2) if t1 == t2 => Some(t1),
            (concrete, Type::UntypedInt(val)) => {
                if Type::UntypedInt(val).try_unify_literal(&concrete) {
                    rhs.ty = Some(concrete.clone());
                    Some(concrete)
                } else {
                    self.creat_compiler_error(format!("Literal {} does not fit into type {:?}", val, concrete), rhs.span);
                    Some(Type::Error)
                }
            },
            (concrete, Type::UntypedFloat(val)) => {
                if concrete.is_float() {
                    rhs.ty = Some(concrete.clone());
                    Some(concrete)
                } else {
                    self.creat_compiler_error(format!("Float {} vs Non-Float {:?}", val, concrete), rhs.span);
                    Some(Type::Error)
                }
            },

            (Type::UntypedInt(val), concrete) => {
                if Type::UntypedInt(val).try_unify_literal(&concrete) {
                    lhs.ty = Some(concrete.clone());
                    Some(concrete)
                } else {
                    self.creat_compiler_error(format!("Literal {} does not fit into type {:?}", val, concrete), lhs.span);
                    Some(Type::Error)
                }
            },
            (Type::UntypedFloat(val), concrete) => {
                if concrete.is_float() {
                    lhs.ty = Some(concrete.clone());
                    Some(concrete)
                } else {
                    self.creat_compiler_error(format!("Float {} vs Non-Float {:?}", val, concrete), lhs.span);
                    Some(Type::Error)
                }
            },

            (Type::Pointer(l_inner), Type::Pointer(r_inner)) => {
                if l_inner == r_inner {
                    Some(Type::Pointer(l_inner))
                } else if l_inner.is_void() || r_inner.is_void() {
                    Some(Type::Pointer(Box::new(Type::Void)))
                } else {
                    None
                }
            },

            (_, _) => None
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
                let l_ty = lhs.ty.as_ref().unwrap();
                let r_ty = rhs.ty.as_ref().unwrap();
                match op {
                    BinaryOp::Add | BinaryOp::Sub => {
                        if let Type::Pointer(inner) = l_ty {
                            if r_ty.is_integer() || matches!(r_ty, Type::UntypedInt(_)) {
                                expr.ty = Some(Type::Pointer(inner.clone()));
                                return;
                            }
                        }
                        if matches!(op, BinaryOp::Sub) {
                            if let (Type::Pointer(t1), Type::Pointer(t2)) = (l_ty, r_ty) {
                                if t1 == t2 {
                                    expr.ty = Some(Type::Usize);
                                    return;
                                }
                            }
                        }

                        if let Some(common) = self.resolve_common_type(lhs, rhs, *op) {
                            if common.is_numeric() || matches!(common, Type::UntypedInt(_) | Type::UntypedFloat(_)) {
                                expr.ty = Some(common);
                            } else {
                                self.creat_compiler_error(format!("Arithmetic not allowed on {:?}", common), expr.span);
                                expr.ty = Some(Type::Error);
                            }
                        } else {
                            self.creat_compiler_error(format!("Type Mismatch: {:?} {:?} {:?}", lhs.ty, op, rhs.ty), expr.span);
                            expr.ty = Some(Type::Error);
                        }
                    },
                    BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        if let Some(common) = self.resolve_common_type(lhs, rhs, *op) {
                            if common.is_numeric() || matches!(common, Type::UntypedInt(_) | Type::UntypedFloat(_)) {
                                expr.ty = Some(common);
                            } else {
                                self.creat_compiler_error(format!("Math op {:?} requires numbers, found {:?}", op, common), expr.span);
                                expr.ty = Some(Type::Error);
                            }
                        } else {
                            self.creat_compiler_error(format!("Type Mismatch: {:?} {:?} {:?}", lhs.ty, op, rhs.ty), expr.span);
                            expr.ty = Some(Type::Error);
                        }
                    },
                    BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => {
                        if let Some(common) = self.resolve_common_type(lhs, rhs, *op) {
                            if common.is_integer() || matches!(common, Type::UntypedInt(_)) {
                                expr.ty = Some(common);
                            } else {
                                self.creat_compiler_error(format!("Bitwise op {:?} requires integers, found {:?}", op, common), expr.span);
                                expr.ty = Some(Type::Error);
                            }
                        } else {
                            self.creat_compiler_error(format!("Type Mismatch: {:?} {:?} {:?}", lhs.ty, op, rhs.ty), expr.span);
                            expr.ty = Some(Type::Error);
                        }
                    },
                    BinaryOp::Shl | BinaryOp::Shr => {
                        if !l_ty.is_integer() && !matches!(l_ty, Type::UntypedInt(_)) {
                            self.creat_compiler_error(format!("Shift target must be integer, found {:?}", l_ty), lhs.span);
                        }

                        if !r_ty.is_integer() && !matches!(r_ty, Type::UntypedInt(_)) {
                            self.creat_compiler_error(format!("Shift amount must be integer, found {:?}", r_ty), rhs.span);
                        }

                        if let (Type::UntypedInt(l), Type::UntypedInt(r)) = (l_ty, r_ty) {
                            let res = match op {
                                BinaryOp::Shl => l << r,
                                BinaryOp::Shr => l >> r,
                                _ => 0
                            };
                            expr.ty = Some(Type::UntypedInt(res));
                        } else {
                            expr.ty = Some(l_ty.clone());
                        }
                    },
                    BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge=> {
                        if let Some(common) = self.resolve_common_type(lhs, rhs, *op) {
                            let valid = common.is_numeric()
                                || common.is_pointer()
                                || matches!(common, Type::BOOL | Type::Enum(_) | Type::UntypedInt(_) | Type::UntypedFloat(_));

                            if !valid {
                                self.creat_compiler_error(format!("Cannot compare types of {:?}", common), expr.span);
                            }

                            if matches!(op, BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge) {
                                if !common.is_numeric() && !common.is_pointer() && !matches!(common, Type::UntypedInt(_) | Type::UntypedFloat(_)) {
                                    self.creat_compiler_error(format!("Cannot order non-numeric types {:?}", common), expr.span);
                                }
                            }

                            expr.ty = Some(Type::BOOL);
                        } else {
                            self.creat_compiler_error(format!("Type Mismatch in comparison: {:?} vs {:?}", lhs.ty, rhs.ty), expr.span);
                            expr.ty = Some(Type::Error);
                        }
                    },
                    BinaryOp::And | BinaryOp::Or => {
                        if l_ty != &Type::BOOL { self.creat_compiler_error(format!("Expected Bool, found {:?}", l_ty), lhs.span); }
                        if r_ty != &Type::BOOL { self.creat_compiler_error(format!("Expected Bool, found {:?}", r_ty), rhs.span); }

                        expr.ty = Some(Type::BOOL);
                    },
                }
            },
            Expr::Unary {op, rhs} => expr.ty = {
                self.visit_expr(rhs);
                let rhs_ty = rhs.ty.as_ref().unwrap();

                match op {
                    UnaryOp::Neg => match rhs_ty {
                        Type::UntypedInt(val) => Some(Type::UntypedInt(-val)),
                        Type::UntypedFloat(val) => Some(Type::UntypedFloat(-val)),
                        t if t.is_numeric() => { Some(t.clone()) },
                        _ => {
                            self.creat_compiler_error(
                                format!("Cannot apply unary minus '-' to type {:?}", rhs_ty),
                                expr.span
                            );
                            Some(Type::Error)
                        }
                    },
                    UnaryOp::Not => {
                        if *rhs_ty == Type::BOOL {
                            Some(Type::BOOL)
                        } else {
                            self.creat_compiler_error(
                                format!("Logical NOT '!' requires boolean, found {:?}", rhs_ty),
                                expr.span
                            );
                            Some(Type::Error)
                        }
                    }
                    UnaryOp::Deref => {
                        if let Type::Pointer(inner) = rhs_ty {
                            if inner.is_void() {
                                self.creat_compiler_error(
                                    "Cannot dereference a void pointer (*void). Cast it first.".into(),
                                    expr.span
                                );
                                Some(Type::Error)
                            } else {
                                Some(*inner.clone())
                            }
                        } else {
                            self.creat_compiler_error(
                                format!("Cannot dereference non-pointer type {:?}", rhs_ty),
                                expr.span
                            );
                            Some(Type::Error)
                        }
                    }
                    UnaryOp::AddressOf => {
                        if !rhs.is_lvalue() {
                            self.creat_compiler_error(
                                "Cannot take address of a temporary value or literal".into(),
                                expr.span
                            );
                        }
                        Some(Type::Pointer(Box::new(rhs_ty.clone())))
                    }
                }
            },
            Expr::LiteralInt(n) => expr.ty = Some(Type::UntypedInt(*n as i128)),
            Expr::LiteralFloat(f) => expr.ty = Some(Type::UntypedFloat(*f)),
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
                match target {
                    SizeOfTarget::Type(type_spec) => {
                        let resolved_ty = self.resolve_type(type_spec);

                        if resolved_ty.is_void() {
                            self.creat_compiler_error("Cannot determine size of 'void'".into(), expr.span);
                        }
                    },
                    SizeOfTarget::Expr(target_expr) => {
                        self.visit_expr(target_expr);
                        let ty = target_expr.ty.as_ref().unwrap();
                        if ty.is_void() {
                            self.creat_compiler_error("Cannot determine size of void expression".into(), expr.span);
                        }
                    }
                }
                expr.ty = Some(Type::Usize);
            },
            Expr::Cast{expr: sub_expr , target} => {
                let dest_ty = self.resolve_type(target);
                self.visit_expr(sub_expr);
                let src_ty = sub_expr.ty.as_ref().unwrap();

                match src_ty.check_cast_safety(&dest_ty) {
                    CastSafety::Forbidden => {
                        self.creat_compiler_error(
                            format!("Invalid cast: Cannot cast from {:?} to {:?}", src_ty, dest_ty),
                            expr.span
                        );
                        expr.ty = Some(Type::Error);
                    },
                    _ => {
                        expr.ty = Some(dest_ty);
                    }
                }
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
                        self.creat_compiler_error(format!("Arg count mismatch: expected {}, got {}", params.len(), args.len()), expr.span);
                    }

                    let limit = std::cmp::min(args.len(), params.len());

                    for i in 0..limit {
                        self.visit_expr(&mut args[i]);
                        let arg_ty = args[i].ty.as_ref().unwrap().clone();
                        let param_ty = &params[i];

                        let is_compatible = match arg_ty {
                            Type::UntypedInt(val) => {
                                if Type::UntypedInt(val).try_unify_literal(param_ty) {
                                    args[i].ty = Some(param_ty.clone());
                                    true
                                } else { false }
                            },
                            Type::UntypedFloat(val) => {
                                if Type::UntypedFloat(val).try_unify_literal(param_ty) {
                                    args[i].ty = Some(param_ty.clone());
                                    true
                                } else { false }
                            },
                            _ => false
                        };

                        if !is_compatible {
                            self.check_assignment(param_ty, &arg_ty, args[i].span)
                        }
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
                    let f_expr_ty = f_expr.ty.as_ref().unwrap().clone();
                    match struct_fields.get(f_name) {
                        Some(f_target_ty) => {
                            let compatible = match f_expr_ty {
                                Type::UntypedInt(val) => {
                                    if Type::UntypedInt(val).try_unify_literal(f_target_ty) {
                                        f_expr.ty = Some(f_target_ty.clone());
                                        true
                                    } else { false }
                                },
                                Type::UntypedFloat(val) => {
                                    if Type::UntypedFloat(val).try_unify_literal(f_target_ty) {
                                        f_expr.ty = Some(f_target_ty.clone());
                                        true
                                    } else { false }
                                },
                                _ => {
                                    self.check_assignment(f_target_ty, &f_expr_ty, f_expr.span);
                                    true
                                }
                            };
                            if !compatible {
                                self.check_assignment(f_target_ty, &f_expr_ty, f_expr.span);
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

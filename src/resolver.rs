use std::collections::HashSet;
use crate::ast::*;
use crate::error::{ Span, CompilerError };
use crate::visitor::ASTVisitor;
use crate::symbols::*;

pub struct SymbolResolver<'a> {
    pub symbols: SymbolTable,
    pub errors: Vec<CompilerError<'a>>,
    pub source: &'a String,
    pub in_loop: bool,
    pub is_assigment_target: bool,
}

impl<'a> SymbolResolver<'a> {
    pub fn new(source: &'a String) -> Self {
        Self {
            symbols: SymbolTable::new(),
            errors: vec![],
            source,
            in_loop: false,
            is_assigment_target: false,
        }
    }

    fn report_error(&mut self, msg: String, span: Span) {
        self.errors.push(CompilerError::new(msg, span, self.source));
    }

    fn define(&mut self, name: String, sym: Symbol, span: Span) {
        if let Err(prev_span) = self.symbols.define(name.clone(), sym) {
            self.report_error(
                format!("Symbol '{}' is already defined. Previous definition at line {}", name, prev_span.line),
                span,
            );
        }
    }

    fn resolve_type(&mut self, ty: & mut TypeSpec, span: Span) {
        match &ty {
            TypeSpec::Named(name) => {
                match self.symbols.resolve(name.clone()) {
                    Some(sym_id) => {
                        let sym = self.symbols.get_symbol(sym_id);
                        match sym.kind {
                            SymbolKind::Fn => self.report_error(format!("Cannot use function as a type: '{}'", name), span),
                            SymbolKind::Var { is_mutable: _ } => self.report_error(format!("Cannot use variable name as a type: '{}'", name), span),
                            SymbolKind::Enum | SymbolKind::Struct  => {}
                        }
                    },
                    None => self.report_error(format!("Unknown type '{:?}'", ty), span)
                }
            },
            TypeSpec::Pointer(pty) => {
                self.resolve_type(&mut *pty.clone(), span);
            },
            TypeSpec::Array(pty, expr) => {
                self.resolve_type(&mut *pty.clone(), span);
                self.visit_expr(&mut *expr.clone());
            },
            TypeSpec::Fn { params, ret } => {
                for param in params {
                    self.resolve_type(&mut param.clone(), span);
                }
                self.resolve_type(&mut *ret.clone(), span);
            },
            _ => {}
        }
    }
}

impl<'a> ASTVisitor<()> for SymbolResolver<'a> {
    fn visit_program(&mut self, program: &mut Program) {
        for item in &mut program.modules {
            self.visit_item(item);
        }
    }

    fn visit_item(&mut self, item: &mut ItemNode) {
        match &mut item.kind {
            Item::Function(f) => self.visit_fn_decl(f),
            Item::Struct(s) => self.visit_struct_decl(s),
            Item::Enum(e) => self.visit_enum_decl(e),
            Item::Extern(e) => self.visit_extern_decl(e),
            Item::Global(g) => self.visit_global_decl(g),
            Item::Import(_) => {},
        }
    }

    fn visit_fn_decl(&mut self, decl: &mut FnDecl) {
        let mut param_symbols: HashSet<String> = HashSet::new();
        let fn_ty = TypeSpec::Fn {
            params: decl.params.iter().map(|(_, t)| t.clone()).collect(),
            ret: Box::new(decl.return_type.clone()),
        };

        self.define(decl.name.clone(), Symbol {
            id: 0,
            name: decl.name.clone(),
            kind: SymbolKind::Fn,
            ty: fn_ty,
            span: decl.span,
            is_initialized: true,
        }, decl.span);

        decl.scope_id = Some(self.symbols.enter_scope());

        for (name, ty) in &decl.params {
            if param_symbols.contains(name) {
                self.report_error(format!("Param '{}' is already defined in extern function '{}'", name, decl.name), decl.span);
            } else {
                param_symbols.insert(name.clone());
            }
            self.define(name.clone(), Symbol {
                id: 0,
                name: name.clone(),
                kind: SymbolKind::Var { is_mutable: false },
                ty: ty.clone(),
                span: decl.span,
                is_initialized: true,
            }, decl.span);
        }

        self.visit_stmt(&mut decl.body);

        self.symbols.exit_scope();
    }

    fn visit_struct_decl(&mut self, decl: &mut StructDecl) {
        let mut field_symbols: HashSet<String> = HashSet::new();
        self.define(decl.name.clone(), Symbol {
            id: 0,
            name: decl.name.clone(),
            kind: SymbolKind::Struct,
            ty: TypeSpec::Named(decl.name.clone()),
            span: decl.span,
            is_initialized: true,
        }, decl.span);
        for (name, _) in &decl.fields {
            if field_symbols.contains(name) {
                self.report_error(format!("Field '{}' is already declared in struct '{}'", name, decl.name), decl.span);
            } else {
                field_symbols.insert(name.clone());
            }
        }
    }

    fn visit_enum_decl(&mut self, decl: &mut EnumDecl) {
        let mut variant_symbols: HashSet<String> = HashSet::new();
        self.define(decl.name.clone(), Symbol {
            id: 0,
            name: decl.name.clone(),
            kind: SymbolKind::Enum,
            ty: TypeSpec::Named(decl.name.clone()),
            span: decl.span,
            is_initialized: true,
        }, decl.span);
        for (name, _) in &decl.variants {
            if variant_symbols.contains(name) {
                self.report_error(format!("Variant '{}' is already defined in enum '{}'", name, decl.name), decl.span);
            } else {
                variant_symbols.insert(name.clone());
            }
        }
    }

    fn visit_extern_decl(&mut self, decl: &mut ExternDecl) {
        let mut param_symbols: HashSet<String> = HashSet::new();

        let fn_ty = TypeSpec::Fn {
            params: decl.params.iter().map(|(_, t)| t.clone()).collect(),
            ret: Box::new(decl.return_type.clone()),
        };
        self.define(decl.name.clone(), Symbol {
            id: 0,
            name: decl.name.clone(),
            kind: SymbolKind::Fn,
            ty: fn_ty,
            span: decl.span,
            is_initialized: false,
        }, decl.span);

        for (name, _) in &decl.params {
            if param_symbols.contains(name) {
                self.report_error(format!("Param '{}' is already defined in extern function '{}'", name, decl.name), decl.span);
            } else {
                param_symbols.insert(name.clone());
            }
        }
    }

    fn visit_global_decl(&mut self, decl: &mut GlobalDecl) {
        self.define(decl.name.clone(), Symbol {
            id: 0,
            name: decl.name.clone(),
            kind: SymbolKind::Var { is_mutable: !decl.is_const },
            ty: decl.ty.clone(),
            span: decl.span,
            is_initialized: decl.init.is_some(),
        }, decl.span);

        if let Some(init) = &mut decl.init {
            self.visit_expr(init);
        }
    }

    fn visit_stmt(&mut self, stmt: &mut StmtNode) {
        match &mut stmt.kind {
            Stmt::Block(stmts, scope_id) => {
                *scope_id = Some(self.symbols.enter_scope());
                for s in stmts {
                    self.visit_stmt(s);
                }
                self.symbols.exit_scope();
            },
            Stmt::VarDecl { is_mutable, name, ty, init } => {
                self.resolve_type(ty, stmt.span);
                if let Some(expr) = init {
                    self.visit_expr(expr);
                }
                self.define(name.clone(), Symbol {
                    id: 0,
                    name: name.clone(),
                    kind: SymbolKind::Var { is_mutable: *is_mutable },
                    ty: ty.clone(),
                    span: stmt.span,
                    is_initialized: init.is_none(),
                }, stmt.span);
            },
            Stmt::Assign { target, value } => {
                self.is_assigment_target = true;
                self.visit_expr(target);
                self.is_assigment_target = false;
                self.visit_expr(value);
            },
            Stmt::If { condition, then_branch, else_branch } => {
                self.visit_expr(condition);
                self.visit_stmt(then_branch);
                if let Some(else_b) = else_branch {
                    self.visit_stmt(else_b);
                }
            },
            Stmt::While { condition, body } => {
                self.visit_expr(condition);
                let in_loop = self.in_loop;
                self.in_loop = true;
                self.visit_stmt(body);
                self.in_loop = in_loop;
            },
            Stmt::For { init, condition, update, body, scope_id } => {
                *scope_id = Some(self.symbols.enter_scope());

                if let Some(i) = init { self.visit_stmt(i); }
                if let Some(c) = condition { self.visit_expr(c); }
                if let Some(u) = update { self.visit_stmt(u); }

                let in_loop = self.in_loop;
                self.in_loop = true;
                self.visit_stmt(body);
                self.in_loop = in_loop;

                self.symbols.exit_scope();
            },
            Stmt::Return(opt_e) => {
                if let Some(e) = opt_e { self.visit_expr(e); }
            },
            Stmt::Expression(e) => { self.visit_expr(e); },
            Stmt::Break => {
                if !self.in_loop {
                    self.report_error("break statement used outside of a loop".to_string(), stmt.span);
                }
            },
            Stmt::Continue => {
                if !self.in_loop {
                    self.report_error("continue statement used outside of a loop".to_string(), stmt.span);
                }
            },
            _ => {}
        }
    }

    fn visit_expr(&mut self, expr: &mut ExprNode) {
        match &mut expr.kind {
            Expr::Identifier(name) => {
                if let Some(sym_id) = self.symbols.resolve(name.clone()) {
                    let sym = self.symbols.get_symbol(sym_id);
                    if self.is_assigment_target {
                        if let SymbolKind::Var { is_mutable } = sym.kind {
                            if !is_mutable {
                                self.report_error(
                                    format!("Cannot assign to immutable variable '{}'. Use 'var' instead of 'let'", name),
                                    expr.span
                                );
                            }
                        } else {
                            self.report_error(format!("Cannot assign to '{}' it's not a variable", name), expr.span);
                        }
                    }
                } else {
                    self.report_error(
                        format!("Undefined variable '{}'", name),
                        expr.span,
                    );
                }
            },
            Expr::Binary { lhs, rhs, .. } => {
                self.visit_expr(lhs);
                self.visit_expr(rhs);
            },
            Expr::Unary { rhs, .. } => {
                self.visit_expr(rhs);
            },
            Expr::Call { callee, args } => {
                self.visit_expr(callee);
                for arg in args {
                    self.visit_expr(arg);
                }
            },
            Expr::MemberAccess { object, is_arrow, .. } => {
                if self.is_assigment_target {
                    if *is_arrow {
                        let was_target = self.is_assigment_target;
                        self.is_assigment_target = false;
                        self.visit_expr(object);
                        self.is_assigment_target = was_target;
                    } else {
                        self.visit_expr(object);
                    }
                } else {
                    self.visit_expr(object);
                }
            },
            Expr::Index { array, index } => {
                self.visit_expr(array);
                let was_target = self.is_assigment_target;
                self.is_assigment_target = false;
                self.visit_expr(index);
                self.is_assigment_target = was_target;
            },
            Expr::StructInit { name, fields } => {
                if let Some(sym_id) = self.symbols.resolve(name.clone()) {
                    let sym = self.symbols.get_symbol(sym_id);
                    if SymbolKind::Struct != sym.kind {
                        self.report_error(
                            format!("Undefined struct '{}'", name), // TODO: better message
                            expr.span,
                        );
                    }
                } else {
                    self.report_error(
                        format!("Undefined struct '{}'", name),
                        expr.span,
                    );
                }
                let mut field_symbols: HashSet<String> = HashSet::new();
                for (fname, e) in fields {
                    if field_symbols.contains(fname) {
                        self.report_error(format!("Field '{}' is already declared in struct initialization '{}'", fname, name), expr.span);
                    } else {
                        field_symbols.insert(fname.clone());
                    }
                    self.visit_expr(e);
                }
            },
            Expr::Cast { expr: e, .. } => self.visit_expr(e),
            Expr::SizeOf { target } => {
                if let SizeOfTarget::Expr(e) = target {
                    self.visit_expr(e);
                }
            }
            _ => {}
        }
    }
}
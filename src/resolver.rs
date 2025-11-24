use crate::ast::*;
use crate::visitor::ASTVisitor;
use crate::symbols::*;

pub struct SymbolResolver {
    pub symbols: SymbolTable,
    pub errors: Vec<String>,
}

impl SymbolResolver {
    pub fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            errors: vec![],
        }
    }

    // Helper to log errors
    fn error(&mut self, msg: String) {
        self.errors.push(msg);
    }
}

impl ASTVisitor<()> for SymbolResolver {
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
            Item::Import(_) => {}, // Imports logic omitted for brevity
        }
    }

    fn visit_fn_decl(&mut self, decl: &mut FnDecl) {
        let fn_ty = Type::Fn {
            params: decl.params.iter().map(|(_, t)| t.clone()).collect(),
            ret: Box::new(decl.return_type.clone()),
        };

        if let Err(e) = self.symbols.define(decl.name.clone(), Symbol {
            name: decl.name.clone(),
            kind: SymbolKind::Fn,
            ty: fn_ty,
        }) {
            self.errors.push(e);
        }

        self.symbols.enter_scope();

        for (name, ty) in &decl.params {
            if let Err(e) = self.symbols.define(name.clone(), Symbol {
                name: name.clone(),
                kind: SymbolKind::Var { is_mutable: false }, // Params usually immutable
                ty: ty.clone(),
            }) {
                self.errors.push(e);
            }
        }

        self.visit_stmt(&mut decl.body);

        self.symbols.exit_scope();
    }

    fn visit_global_decl(&mut self, decl: &mut GlobalDecl) {
        if let Err(e) = self.symbols.define(decl.name.clone(), Symbol {
            name: decl.name.clone(),
            kind: SymbolKind::Var { is_mutable: !decl.is_const },
            ty: decl.ty.clone(),
        }) {
            self.errors.push(e);
        }

        if let Some(init) = &mut decl.init {
            self.visit_expr(init);
        }
    }

    fn visit_struct_decl(&mut self, decl: &mut StructDecl) {
        // Simple registration of the struct name
        if let Err(e) = self.symbols.define(decl.name.clone(), Symbol {
            name: decl.name.clone(),
            kind: SymbolKind::Struct,
            ty: Type::Named(decl.name.clone()),
        }) {
            self.errors.push(e);
        }
        // TODO: You might want to store fields in a separate "TypeEnv" later
    }

    fn visit_enum_decl(&mut self, decl: &mut EnumDecl) {
        self.symbols.define(decl.name.clone(), Symbol {
            name: decl.name.clone(),
            kind: SymbolKind::Enum,
            ty: Type::Named(decl.name.clone()),
        });
    }

    fn visit_extern_decl(&mut self, decl: &mut ExternDecl) {
        let fn_ty = Type::Fn {
            params: decl.params.iter().map(|(_, t)| t.clone()).collect(),
            ret: Box::new(decl.return_type.clone()),
        };
        self.symbols.define(decl.name.clone(), Symbol {
            name: decl.name.clone(),
            kind: SymbolKind::Fn,
            ty: fn_ty,
        });
    }

    fn visit_stmt(&mut self, stmt: &mut StmtNode) {
        match &mut stmt.kind {
            Stmt::Block(stmts) => {
                self.symbols.enter_scope();
                for s in stmts {
                    self.visit_stmt(s);
                }
                self.symbols.exit_scope();
            },
            Stmt::VarDecl { is_mutable, name, ty, init } => {
                if let Some(expr) = init {
                    self.visit_expr(expr);
                }
                self.symbols.define(name.clone(), Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Var { is_mutable: *is_mutable },
                    ty: ty.clone(),
                });
            },
            Stmt::Assign { target, value } => {
                self.visit_expr(target);
                self.visit_expr(value);
            },
            Stmt::If { condition, then_branch, else_branch } => {
                self.visit_expr(condition);
                // Note: Block stmts handle their own scoping, so we just visit
                self.visit_stmt(then_branch);
                if let Some(else_b) = else_branch {
                    self.visit_stmt(else_b);
                }
            },
            Stmt::While { condition, body } => {
                self.visit_expr(condition);
                self.visit_stmt(body);
            },
            Stmt::For { init, condition, update, body } => {
                self.symbols.enter_scope(); // Implicit scope for loop var

                if let Some(i) = init { self.visit_stmt(i); }
                if let Some(c) = condition { self.visit_expr(c); }
                if let Some(u) = update { self.visit_stmt(u); }

                self.visit_stmt(body);

                self.symbols.exit_scope();
            },
            Stmt::Return(opt_e) => {
                if let Some(e) = opt_e { self.visit_expr(e); }
            },
            Stmt::Expression(e) => { self.visit_expr(e); },
            _ => {} // Break, Continue
        }
    }

    fn visit_expr(&mut self, expr: &mut ExprNode) {
        match &mut expr.kind {
            Expr::Identifier(name) => {
                if let Some(sym) = self.symbols.resolve(name) {
                    // Success! We found it.
                    // We could optionally verify mutability here if we passed context.
                } else {
                    self.errors.push(format!("Undefined variable '{}'", name));
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
            Expr::MemberAccess { object, .. } => {
                self.visit_expr(object);
                // Member resolution usually happens in TypeCheck, not SymbolResolve
            },
            Expr::Index { array, index } => {
                self.visit_expr(array);
                self.visit_expr(index);
            },
            Expr::StructInit { fields, .. } => {
                for (_, e) in fields {
                    self.visit_expr(e);
                }
            },
            Expr::Cast { expr: e, .. } => self.visit_expr(e),
            Expr::SizeOf { target } => {
                if let SizeOfTarget::Expr(e) = target {
                    self.visit_expr(e);
                }
            }
            _ => {} // Literals, Null
        }
    }
}
use std::collections::{HashMap, HashSet};
use crate::ast::*;
use crate::error::{CompilerError, Span};
use crate::type_registry::*;
use crate::type_resolver::TypeResolver;
use crate::visitor::ASTVisitor;

pub struct TypeCollector<'a> {
    pub registry: TypeRegistry,
    type_resolver: TypeResolver,
    file_source: &'a String,
    pub errors: Vec<CompilerError<'a>>,
}

impl<'a> TypeCollector<'a> {
    pub fn new(file_source: &'a String) -> Self {
        Self {
            registry: TypeRegistry::new(),
            type_resolver: TypeResolver::new(),
            file_source,
            errors: vec![],
        }
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
}

impl<'a> ASTVisitor<()> for TypeCollector<'a> {
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
            _ => {},
        }
    }

    fn visit_fn_decl(&mut self, decl: &mut FnDecl) -> () {
        let mut params: Vec<(String, Type)> = vec![];
        for (name, ty) in &decl.params {
            match self.type_resolver.resolve_type(ty) {
                Ok(res_ty) => params.push((name.clone(), res_ty)),
                Err((msg, span)) => self.creat_compiler_error(msg, span)
            }
        }
        let mut return_ty = Type::Error;
        match self.type_resolver.resolve_type(&decl.return_type) {
            Ok(res_ty) => return_ty = res_ty,
            Err((msg, span)) => self.creat_compiler_error(msg, span)
        }
        self.registry.register_fn(decl.name.clone(), params, return_ty);
    }

    fn visit_struct_decl(&mut self, decl: &mut StructDecl) -> () {
        let mut fields: Vec<(String, Type)> = vec![];
        for (name, ty) in &decl.fields {
            match self.type_resolver.resolve_type(ty) {
                Ok(res_ty) => fields.push((name.clone(), res_ty)),
                Err((msg, span)) => self.creat_compiler_error(msg, span)
            }
        }
        self.registry.register_struct(decl.name.clone(), fields);
    }

    fn visit_enum_decl(&mut self, decl: &mut EnumDecl) -> () {
        let mut variants: Vec<(String, i64)> = vec![];
        let mut next_discriminant:i64 = 0;

        let mut min_val: i64 = 0;
        let mut max_val: i64 = 0;
        let mut first_variant = true;

        let mut used_values = HashSet::new();
        for (name, value_expr) in &decl.variants {
            let current_val = match value_expr {
                None => {
                    let val = next_discriminant;
                    next_discriminant += 1;
                    val
                }
                Some(expr) => {
                    match self.type_resolver.eval(expr) {
                        None => {
                            self.creat_compiler_error("Enum value must be a constant integer".into(), decl.span);
                            0
                        }
                        Some(val) => {
                            next_discriminant = (val as i64) + 1;
                            val as i64
                        }
                    }
                }
            };

            if used_values.contains(&current_val) {
                self.creat_compiler_warning(
                    format!("Discriminant value {} is used multiple times", current_val),
                    decl.span
                );
            }
            used_values.insert(current_val);
            if first_variant {
                min_val = current_val;
                max_val = current_val;
                first_variant = false;
            } else {
                if current_val < min_val { min_val = current_val; }
                if current_val > max_val { max_val = current_val; }
            }
            variants.push((name.clone(), current_val));
        }
        let backing_type = TypeResolver::determine_backing_type(min_val, max_val);
        self.registry.register_enum(decl.name.clone(), variants, backing_type);
    }

    fn visit_extern_decl(&mut self, decl: &mut ExternDecl) -> () {
        let mut params: Vec<(String, Type)> = vec![];
        for (name, ty) in &decl.params {
            match self.type_resolver.resolve_type(ty) {
                Ok(res_ty) => params.push((name.clone(), res_ty)),
                Err((msg, span)) => self.creat_compiler_error(msg, span)
            }
        }
        let mut return_ty = Type::Error;
        match self.type_resolver.resolve_type(&decl.return_type) {
            Ok(res_ty) => return_ty = res_ty,
            Err((msg, span)) => self.creat_compiler_error(msg, span)
        }
        self.registry.register_fn(decl.name.clone(), params, return_ty);
    }

    fn visit_global_decl(&mut self, decl: &mut GlobalDecl) -> () {
        unreachable!()
    }

    fn visit_stmt(&mut self, stmt: &mut StmtNode) -> () {
        unreachable!()
    }

    fn visit_expr(&mut self, expr: &mut ExprNode) -> () {
        unreachable!()
    }
}
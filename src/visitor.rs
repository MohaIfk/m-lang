use crate::ast::*;

struct SemanticError {}

pub trait ASTVisitor<V> {
    fn visit_program(&mut self, program: &mut Program) -> V;
    fn visit_item(&mut self, item: &mut ItemNode) -> V;

    fn visit_fn_decl(&mut self, decl: &mut FnDecl) -> V;
    fn visit_struct_decl(&mut self, decl: &mut StructDecl) -> V;
    fn visit_enum_decl(&mut self, decl: &mut EnumDecl) -> V;
    fn visit_extern_decl(&mut self, decl: &mut ExternDecl) -> V;
    fn visit_global_decl(&mut self, decl: &mut GlobalDecl) -> V;

    fn visit_stmt(&mut self, stmt: &mut StmtNode) -> V;
    // fn visit_block(&mut self, block: &mut StmtNode) -> V;

    fn visit_expr(&mut self, expr: &mut ExprNode) -> V;
}

use std::collections::HashMap;
use crate::ast::*;
use crate::error::Span;

#[derive(Debug, PartialEq)]
pub enum SymbolKind {
    Var { is_mutable: bool },
    Fn,
    Struct,
    Enum,
}

#[derive(Debug)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub ty: Type,
    pub span: Span,
}

pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self { scopes: vec![HashMap::new()] }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: String, sym: Symbol) -> Result<(), Span> {
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(prev) = scope.get(&name) {
                return Err(prev.span);
            }
            scope.insert(name, sym);
            Ok(())
        } else {
            panic!("Symbol table corrupted: no scopes");
        }
    }

    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(sym);
            }
        }
        None
    }
}
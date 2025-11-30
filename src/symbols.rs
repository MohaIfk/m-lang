use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::ast::*;
use crate::error::Span;

pub type SymbolId = usize;
pub type ScopeId = usize;

#[derive(Debug, PartialEq)]
pub enum SymbolKind {
    Var { is_mutable: bool },
    Fn,
    Struct,
    Enum,
}

#[derive(Debug)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub ty: TypeSpec,
    pub span: Span,
    pub is_initialized: bool, // for variables, it means had a value and for struct, enum, fn it means if it has definition
}

#[derive(Debug)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub symbols: HashMap<String, SymbolId>,
}

#[derive(Debug)]
pub struct SymbolTable {
    pub scopes: Vec<Scope>,
    pub symbols: Vec<Symbol>,
    active_scope_stack: Vec<ScopeId>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut st = Self { scopes: vec![], symbols: vec![], active_scope_stack: vec![] };
        st.enter_scope();
        st
    }

    pub fn enter_scope(&mut self) -> ScopeId {
        let parent = self.active_scope_stack.last().cloned();
        self.scopes.push(Scope { id: 0, parent, symbols: HashMap::new() });
        let scope_id = self.scopes.len() - 1;
        self.scopes[scope_id].id = scope_id;
        self.active_scope_stack.push(scope_id);
        scope_id
    }

    pub fn exit_scope(&mut self) {
        self.active_scope_stack.pop();
    }

    pub fn current_scope_id(&self) -> ScopeId {
        self.active_scope_stack.last().expect("No active scope").clone()
    }

    pub const fn global_scope_id(&self) -> ScopeId { 0 }

    pub fn define(&mut self, name: String, sym: Symbol) -> Result<SymbolId, Span> {
        let current_scope_id = self.current_scope_id();
        let scope = &mut self.scopes[current_scope_id];
        match scope.symbols.entry(name) {
            Entry::Occupied(entry) => {
                let prev_id = *entry.get();
                let prev_sym = &self.symbols[prev_id];
                Err(prev_sym.span)
            },
            Entry::Vacant(entry) => {
                self.symbols.push(sym);
                let symbol_id = self.symbols.len() - 1;
                entry.insert(symbol_id);
                self.symbols[symbol_id].id = symbol_id;
                Ok(symbol_id)
            }
        }
    }

    pub fn resolve(&self, name: String) -> Option<SymbolId> {
        let mut current_id = *self.active_scope_stack.last()?;
        loop {
            let scope = &self.scopes[current_id];
            match scope.symbols.get(&name) {
                Some(&id) => return Some(id),
                None => match scope.parent {
                    Some(parent_id) => current_id = parent_id,
                    None => return None,
                }
            }
        }
    }

    pub fn resolve_local(&mut self, name: String) -> Option<SymbolId> {
        let current_scope_id = self.current_scope_id();
        let scope = &mut self.scopes[current_scope_id];
        match scope.symbols.entry(name) {
            Entry::Occupied(entry) => Some(entry.get().clone()),
            Entry::Vacant(_) => None,
        }
    }

    pub fn get_symbol(&self, id: SymbolId) -> &Symbol {
        self.symbols.get(id).expect("Symbol not found")
    }

    pub fn get_symbol_mut(&mut self, id: SymbolId) -> &mut Symbol {
        self.symbols.get_mut(id).expect("Symbol not found")
    }
}
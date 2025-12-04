use std::collections::HashMap;
use crate::ast::Type;

pub type TypeId = usize;

#[derive(Debug, Clone)]
pub enum TypeInfo {
    Struct { name: String, fields: Vec<(String, Type)> },
    Enum { name: String, variants: Vec<(String, i64)>, backing_type: Type },
    Fn { name: String, params: Vec<(String, Type)>, return_type: Type},
    Primitive(Type),
}

#[derive(Debug)]
pub struct Layout {
    pub size: usize,
    pub align: usize,
    pub offsets: Vec<usize>,
}

#[derive(Debug)]
pub struct TypeRegistry {
    pub type_names: HashMap<String, TypeId>,
    pub types: Vec<TypeInfo>,
    pub layouts: HashMap<TypeId, Layout>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            type_names: HashMap::new(),
            types: vec![],
            layouts: HashMap::new(),
        }
    }

    pub fn exist(&self, name: &String) -> bool {
        self.type_names.contains_key(name)
    }

    pub fn get_type(&self, name: &String) -> Option<TypeInfo> {
        let id =  self.type_names.get(name)?;
        self.types.get(*id).cloned()
    }

    pub fn register_struct(&mut self, name: String, fields: Vec<(String, Type)>) {
        let type_id = self.types.len();
        self.types.push(TypeInfo::Struct { name: name.clone(), fields });
        self.type_names.insert(name, type_id);
    }

    pub fn register_enum(&mut self, name: String, variants: Vec<(String, i64)>, backing_type: Type) {
        let type_id = self.types.len();
        self.types.push(TypeInfo::Enum { name: name.clone(), variants, backing_type });
        self.type_names.insert(name, type_id);
    }

    pub fn register_fn(&mut self, name: String, params: Vec<(String, Type)>, return_type: Type) {
        let type_id = self.types.len();
        self.types.push(TypeInfo::Fn { name: name.clone(), params, return_type, });
        self.type_names.insert(name, type_id);
    }

    pub fn get(&self, name: String) -> Option<&TypeInfo> {
        let type_id = self.type_names.get(&name)?;
        self.types.get(*type_id)
    }
}
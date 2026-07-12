use std::{collections::HashMap, iter::once};

use serde::Serialize;

use crate::{translatable::Type, type_definition::TypeDefinition};

#[derive(Debug, Serialize)]
pub struct ApiSpec {
    pub name: String,
    pub version: String,
    pub methods: HashMap<String, LiRpcMethodSpec>,
    pub types: HashMap<String, TypeDefinition>,
}

impl ApiSpec {
    /// # Error
    /// will return the names of the types that are referenced in a handler but not present in the list of type definitions.
    pub fn new(
        name: String,
        version: String,
        methods: HashMap<String, LiRpcMethodSpec>,
        types: HashMap<String, TypeDefinition>,
    ) -> Result<Self, Vec<String>> {
        let referenced_types: Vec<&str> = methods
            .values()
            .flat_map(|m| {
                m.messages
                    .iter()
                    .map(|msg| match &msg {
                        Type::TypeRef(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .chain(once(match &m.returns {
                        Type::TypeRef(s) => Some(s.as_str()),
                        _ => None,
                    }))
                    .flatten()
            })
            .collect();

        let no_definitions = referenced_types
            .iter()
            .filter_map(|t| {
                if types.contains_key(*t) {
                    None
                } else {
                    Some(t.to_string())
                }
            })
            .collect::<Vec<String>>();

        if !no_definitions.is_empty() {
            return Err(no_definitions);
        }

        Ok(Self {
            name,
            version,
            methods,
            types,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct LiRpcMethodSpec {
    pub messages: Vec<Type>,
    pub returns: Type,
}

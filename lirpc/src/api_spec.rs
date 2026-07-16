use std::{collections::HashMap, iter::once};

use serde::{Deserialize, Serialize};

use crate::{translatable::Type, type_definition::TypeDefinition};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSpec {
    pub name: String,
    pub version: String,
    pub methods: HashMap<String, LiRpcMethodSpec>,
    pub types: HashMap<String, TypeDefinition>,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiSpecError {
    #[error("invalid api spec JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("the following types are referenced by a method but have no type definition: {0:?}")]
    MissingTypeDefinitions(Vec<String>),
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
        let spec = Self {
            name,
            version,
            methods,
            types,
        };

        spec.validate()?;

        Ok(spec)
    }

    /// Parses an `ApiSpec` from its JSON representation (as produced by
    /// `compile_json_api_spec!`), re-running the same referenced-type validation as [`ApiSpec::new`].
    pub fn from_json(json: &str) -> Result<Self, ApiSpecError> {
        let spec: Self = serde_json::from_str(json)?;

        spec.validate()
            .map_err(ApiSpecError::MissingTypeDefinitions)?;

        Ok(spec)
    }

    fn validate(&self) -> Result<(), Vec<String>> {
        let referenced_types: Vec<&str> = self
            .methods
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
                if self.types.contains_key(*t) {
                    None
                } else {
                    Some(t.to_string())
                }
            })
            .collect::<Vec<String>>();

        if !no_definitions.is_empty() {
            return Err(no_definitions);
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LiRpcMethodSpec {
    pub messages: Vec<Type>,
    pub returns: Type,
}

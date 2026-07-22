use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{translatable::Type, type_definition::TypeDefinition};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSpec {
    pub name: String,
    pub version: String,
    pub methods: BTreeMap<String, LiRpcMethodSpec>,
    pub types: BTreeMap<String, TypeDefinition>,
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
        methods: BTreeMap<String, LiRpcMethodSpec>,
        types: BTreeMap<String, TypeDefinition>,
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

    fn get_type_refs_from_type(ty: &Type) -> Vec<&str> {
        match ty {
            Type::TypeRef(type_ref) => vec![type_ref.as_str()],
            Type::Option(ty) | Type::Box(ty) | Type::Vec(ty) => Self::get_type_refs_from_type(ty),
            Type::Result(ty1, ty2) | Type::HashMap(ty1, ty2) => {
                let mut combined = Self::get_type_refs_from_type(ty1);
                combined.append(&mut Self::get_type_refs_from_type(ty2));

                combined
            }
            _ => Vec::new(),
        }
    }

    fn validate(&self) -> Result<(), Vec<String>> {
        let referenced_types: Vec<&str> = self
            .methods
            .values()
            .flat_map(|m| {
                m.messages
                    .iter()
                    .flat_map(Self::get_type_refs_from_type)
                    .chain(Self::get_type_refs_from_type(&m.returns))
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        api_spec::{ApiSpec, LiRpcMethodSpec},
        translatable::Type,
    };

    #[test]
    fn should_deny_recursive_type_with_type_ref() {
        let api_spec = ApiSpec::new(
            "myapp".to_string(),
            "0.1.0".to_string(),
            BTreeMap::from([(
                "greet".to_string(),
                LiRpcMethodSpec {
                    messages: vec![],
                    returns: Type::Result(
                        Box::new(Type::I128),
                        Box::new(Type::TypeRef("Error".to_string())),
                    ),
                },
            )]),
            BTreeMap::new(),
        );

        assert!(api_spec.is_err());
    }
}

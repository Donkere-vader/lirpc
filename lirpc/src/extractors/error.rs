use serde::{Deserialize, Serialize};

use crate::translatable::{Translatable, Type};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "error", content = "details")]
pub enum LiRpcExtractorError {
    SerdeError(String),
}

impl Translatable for LiRpcExtractorError {
    fn get_type() -> Type {
        Type::TypeRef("LiRpcExtractorError".to_string())
    }
}

impl From<serde_json::Error> for LiRpcExtractorError {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeError(format!("{value}"))
    }
}

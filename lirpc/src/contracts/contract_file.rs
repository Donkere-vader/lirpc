use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::contracts::serializable_type::SerializableType;

#[derive(Serialize, Deserialize)]
pub struct LiRpcType {
    pub fields: HashMap<String, SerializableType>, // field name: field type
}

#[derive(Serialize, Deserialize)]
pub struct LiRpcMethod {
    pub output: Option<SerializableType>,
    pub message: Option<SerializableType>,
}

#[derive(Serialize, Deserialize)]
pub struct ContractFile {
    pub version: String,
    pub types: HashMap<String, LiRpcType>,
    pub methods: HashMap<String, LiRpcMethod>,
}

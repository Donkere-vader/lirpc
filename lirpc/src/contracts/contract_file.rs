use std::collections::HashMap;

use serde::Serialize;

use crate::contracts::serializable_type::SerializableType;

#[derive(Serialize)]
pub struct LiRpcType {
    pub fields: HashMap<String, SerializableType>, // field name: field type
}

#[derive(Serialize)]
pub struct LiRpcMethod {
    pub output: Option<SerializableType>,
    pub message: Option<SerializableType>,
}

#[derive(Serialize)]
pub struct ContractFile {
    pub version: String,
    pub types: HashMap<String, LiRpcType>,
    pub methods: HashMap<String, LiRpcMethod>,
}

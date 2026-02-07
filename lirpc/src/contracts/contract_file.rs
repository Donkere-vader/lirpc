use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::contracts::{lirpc_method_file::LiRpcMethodReturn, serializable_type::SerializableType};

#[derive(Serialize, Deserialize)]
pub struct LiRpcType {
    pub fields: BTreeMap<String, SerializableType>, // field name: field type
}

#[derive(Serialize, Deserialize)]
pub struct LiRpcMethod {
    pub output: Option<SerializableType>,
    pub message: Option<SerializableType>,
    pub return_type: LiRpcMethodReturn,
}

#[derive(Serialize, Deserialize)]
pub struct ContractFile {
    pub version: String,
    pub types: BTreeMap<String, LiRpcType>,
    pub methods: BTreeMap<String, LiRpcMethod>,
}

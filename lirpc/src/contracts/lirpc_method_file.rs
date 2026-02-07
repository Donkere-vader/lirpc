use serde::{Deserialize, Serialize};

use crate::contracts::serializable_type::SerializableType;

#[derive(Serialize, Deserialize)]
pub struct LiRpcMethodFile {
    pub name: String,
    pub output: Option<SerializableType>,
    pub message: Option<SerializableType>,
    pub return_type: LiRpcMethodReturn,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", content = "err_variant")]
pub enum LiRpcMethodReturn {
    None,
    Result(SerializableType),
}

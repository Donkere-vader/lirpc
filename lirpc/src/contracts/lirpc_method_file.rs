use serde::{Deserialize, Serialize};

use crate::contracts::serializable_type::SerializableType;

#[derive(Serialize, Deserialize)]
pub struct LiRpcMethodFile {
    pub name: String,
    pub output: Option<SerializableType>,
    pub message: Option<SerializableType>,
}

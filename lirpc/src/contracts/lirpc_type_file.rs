use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::contracts::serializable_type::SerializableType;

#[derive(Serialize, Deserialize)]
pub struct LiRpcTypeFile {
    pub name: String,
    pub fields: HashMap<String, SerializableType>, // field name: field type
}

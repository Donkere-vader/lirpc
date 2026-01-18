use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::contracts::serializable_type::SerializableType;

#[derive(Serialize, Deserialize)]
pub struct LiRpcTypeFile {
    pub name: String,
    pub fields: BTreeMap<String, SerializableType>, // field name: field type
}

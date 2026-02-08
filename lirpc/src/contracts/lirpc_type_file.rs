use serde::{Deserialize, Serialize};

use crate::contracts::contract_file::LiRpcType;

#[derive(Serialize, Deserialize)]
pub struct LiRpcTypeFile {
    pub name: String,
    pub r#type: LiRpcType,
}

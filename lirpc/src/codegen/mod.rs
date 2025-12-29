pub mod error;

use tokio::fs;

use crate::{codegen::error::CodeGenError, contracts::contract_file::ContractFile};

pub async fn code_gen(
    contract_path: &str,
    _language: &str,
    _output_path: &str,
) -> Result<(), CodeGenError> {
    let contract_contents = fs::read_to_string(contract_path).await?;
    let _contract = serde_json::from_str::<ContractFile>(&contract_contents)?;
    todo!()
}

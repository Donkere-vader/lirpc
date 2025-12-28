use std::time::SystemTime;

use glob::glob;
use tokio::fs;

use crate::contracts::{
    contract_file::{ContractFile, LiRpcMethod, LiRpcType},
    lirpc_method_file::LiRpcMethodFile,
    lirpc_type_file::LiRpcTypeFile,
};

#[derive(thiserror::Error, Debug)]
pub enum CompileError {
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::error::Error),
    #[error("No directory target/[debug / release]/build was found")]
    NoBuildFound,
    #[error("PatternError: {0}")]
    PatternError(#[from] glob::PatternError),
    #[error("GlobError: {0}")]
    GlobError(#[from] glob::GlobError),
    #[error("File was encountered with non UTF-8 file name")]
    InvalidFileName,
    #[error(
        "No build folder could be found that matched the name of the specified crate name and build target"
    )]
    NoBuildFolderFound,
}

pub async fn compile(
    crate_name: &str,
    target_name: &str,
    contract_path: &str,
    version: String,
    release: bool,
    minimal: bool,
) -> Result<(), CompileError> {
    let mut type_files = Vec::new();
    let mut method_files = Vec::new();

    let out_dir = latest_out_dir(crate_name, release).await?;

    println!("out_dir={out_dir}");

    let type_signature_file_glob = format!("{out_dir}/lirpc-{target_name}/type-*.json",);
    let method_signature_file_glob = format!("{out_dir}/lirpc-{target_name}/method-*.json",);

    for entry in glob(&type_signature_file_glob)? {
        let entry_path = entry?;
        let contents = fs::read_to_string(&entry_path).await?;
        type_files.push(serde_json::from_str::<LiRpcTypeFile>(&contents)?);
    }

    for entry in glob(&method_signature_file_glob)? {
        let entry_path = entry?;
        let contents = fs::read_to_string(&entry_path).await?;
        method_files.push(serde_json::from_str::<LiRpcMethodFile>(&contents)?);
    }

    let contract = ContractFile {
        version,
        types: type_files
            .into_iter()
            .map(|t| (t.name, LiRpcType { fields: t.fields }))
            .collect(),
        methods: method_files
            .into_iter()
            .map(|m| {
                (
                    m.name,
                    LiRpcMethod {
                        message: m.message,
                        output: m.output,
                    },
                )
            })
            .collect(),
    };

    let serialized_contract = match minimal {
        true => serde_json::to_string(&contract)?,
        false => serde_json::to_string_pretty(&contract)?,
    };

    fs::write(contract_path, serialized_contract).await?;

    Ok(())
}

async fn latest_out_dir(crate_name: &str, release: bool) -> Result<String, CompileError> {
    let build_folder = format!("target/{}/build", if release { "release" } else { "debug" });

    if !fs::try_exists(&build_folder).await? {
        return Err(CompileError::NoBuildFound);
    }

    let mut latest_modification = SystemTime::UNIX_EPOCH;
    let mut latest_modified = None;
    let mut directory_listing = fs::read_dir(&build_folder).await?;
    while let Some(item) = directory_listing.next_entry().await? {
        let filename = item
            .file_name()
            .to_str()
            .ok_or(CompileError::InvalidFileName)?
            .to_string();

        if !filename.starts_with(crate_name) {
            continue;
        }

        let metadata = item.metadata().await?;

        let modified = metadata.modified()?;
        if modified > latest_modification {
            latest_modification = modified;
            latest_modified = Some(filename)
        }
    }

    latest_modified
        .map(|f| format!("{build_folder}/{f}/out"))
        .ok_or(CompileError::NoBuildFolderFound)
}

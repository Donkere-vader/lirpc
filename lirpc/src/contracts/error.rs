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

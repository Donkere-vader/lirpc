#[derive(Debug, thiserror::Error)]
pub enum CodeGenError {
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::error::Error),
}

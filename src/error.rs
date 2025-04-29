use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZipError {
    #[error("{0}")]
    Error(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

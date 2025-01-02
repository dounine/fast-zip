use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZipError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TodoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Scan error in {file}: {message}")]
    Scan { file: String, message: String },
}

pub type Result<T> = std::result::Result<T, TodoError>;

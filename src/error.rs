use thiserror::Error;

/// Application-specific error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to get home directory")]
    HomeDirectoryNotFound,
    
    #[error("Mutex lock poisoned: {0}")]
    MutexPoisoned(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Invalid timestamp")]
    InvalidTimestamp,
}

/// Result type alias for application errors
pub type AppResult<T> = Result<T, AppError>;
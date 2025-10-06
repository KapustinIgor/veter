//! Error types for Veter core

use thiserror::Error;

pub type Result<T> = std::result::Result<T, VeterError>;

#[derive(Error, Debug)]
pub enum VeterError {
    #[error("Cryptographic error: {0}")]
    Crypto(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Key management error: {0}")]
    KeyManagement(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for VeterError {
    fn from(err: sqlx::Error) -> Self {
        VeterError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for VeterError {
    fn from(err: serde_json::Error) -> Self {
        VeterError::Serialization(err.to_string())
    }
}

impl From<bincode::Error> for VeterError {
    fn from(err: bincode::Error) -> Self {
        VeterError::Serialization(err.to_string())
    }
}

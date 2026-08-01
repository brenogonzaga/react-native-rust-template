use app_core::CoreError;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "lowercase")]
pub enum BridgeError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for BridgeError {
    fn from(err: serde_json::Error) -> Self {
        BridgeError::Serialization(err.to_string())
    }
}

impl From<CoreError> for BridgeError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::UserNotFound(msg) => BridgeError::NotFound(msg),
            CoreError::FactorialOverflow(msg) => {
                BridgeError::InvalidArgument(format!("Factorial input overflow: {}", msg))
            }
        }
    }
}

#[allow(dead_code)]
pub trait ResultExt<T> {
    fn to_bridge(self) -> Result<T, BridgeError>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn to_bridge(self) -> Result<T, BridgeError> {
        self.map_err(|e| BridgeError::Internal(e.to_string()))
    }
}

use app_core::CoreError;
use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;

#[derive(Error, Debug, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum BridgeError {
    #[error("User not found: {id}")]
    NotFound { id: String },

    #[error("Factorial input {n} exceeds maximum supported limit of 20")]
    FactorialOverflow { n: u64 },

    #[error("Invalid argument: {reason}")]
    InvalidArgument { reason: String },

    #[error("Internal error: {reason}")]
    Internal { reason: String },

    #[error("Serialization error: {reason}")]
    Serialization { reason: String },
}

impl From<serde_json::Error> for BridgeError {
    fn from(err: serde_json::Error) -> Self {
        BridgeError::Serialization {
            reason: err.to_string(),
        }
    }
}

impl From<CoreError> for BridgeError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::UserNotFound(id) => BridgeError::NotFound { id },
            CoreError::FactorialOverflow(n) => BridgeError::FactorialOverflow { n },
        }
    }
}

#[allow(dead_code)]
pub trait ResultExt<T> {
    fn to_bridge(self) -> Result<T, BridgeError>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn to_bridge(self) -> Result<T, BridgeError> {
        self.map_err(|e| BridgeError::Internal {
            reason: e.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_serializes_to_kind_plus_params() {
        let err = BridgeError::from(CoreError::UserNotFound("42".into()));
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json, serde_json::json!({ "kind": "not_found", "id": "42" }));
    }

    #[test]
    fn factorial_overflow_serializes_to_kind_plus_params() {
        let err = BridgeError::from(CoreError::FactorialOverflow(21));
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "kind": "factorial_overflow", "n": 21 })
        );
    }
}

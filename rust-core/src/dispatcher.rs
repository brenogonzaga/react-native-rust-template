use crate::error::BridgeError;
use crate::handlers::math::{MathCommand, MathResponses};
use crate::handlers::system::{SystemCommand, SystemResponses};
use crate::handlers::user::{UserCommand, UserResponses};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use ts_rs::TS;

/// Last-resort payload for when even error serialization fails.
const SERIALIZATION_FALLBACK: &str =
    r#"{"status":"error","kind":"internal","reason":"failed to serialize bridge response"}"#;

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum BridgeResponse<T> {
    Success { data: T },
    Error(BridgeError),
}

/// Single producer of the error envelope, so every failure path agrees on its
/// shape.
pub fn err_json(err: BridgeError) -> String {
    tracing::warn!(error = %err, "bridge call failed");
    serde_json::to_string(&BridgeResponse::<()>::Error(err))
        .unwrap_or_else(|_| SERIALIZATION_FALLBACK.to_string())
}

/// Every command the bridge accepts. `callRust()` in TypeScript takes exactly
/// this type, so command names and args are checked against this enum at
/// compile time — the TS type *is* the wire format.
#[derive(Deserialize, TS)]
#[ts(export)]
#[serde(tag = "cmd", content = "args", rename_all = "snake_case")]
pub enum BridgeCommand {
    System(SystemCommand),
    Math(MathCommand),
    User(UserCommand),
}

/// Type-only: the `data` type of every command, as `[cmd][type]`. `callRust()`
/// infers its return type from this, so it can't drift from the Rust side.
#[derive(TS)]
#[ts(export)]
#[allow(dead_code)]
pub struct BridgeResponses {
    pub system: SystemResponses,
    pub math: MathResponses,
    pub user: UserResponses,
}

pub struct BridgeDispatcher;

impl BridgeDispatcher {
    pub fn run(command: BridgeCommand) -> String {
        let result: Result<Box<RawValue>, BridgeError> = match command {
            BridgeCommand::System(cmd) => cmd.dispatch(),
            BridgeCommand::Math(cmd) => cmd.dispatch(),
            BridgeCommand::User(cmd) => cmd.dispatch(),
        };

        match result {
            Ok(data) => serde_json::to_string(&BridgeResponse::Success { data })
                .unwrap_or_else(|_| SERIALIZATION_FALLBACK.to_string()),
            Err(err) => err_json(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(envelope: &str) -> String {
        BridgeDispatcher::run(serde_json::from_str(envelope).unwrap())
    }

    #[test]
    fn success_embeds_handler_json_verbatim() {
        assert_eq!(
            run(r#"{"cmd":"system","args":{"type":"ping"}}"#),
            r#"{"status":"success","data":"pong"}"#
        );
    }

    #[test]
    fn error_carries_kind_and_params_alongside_status() {
        assert_eq!(
            run(r#"{"cmd":"user","args":{"type":"get_user","id":"nope"}}"#),
            r#"{"status":"error","kind":"not_found","id":"nope"}"#
        );
    }

    #[test]
    fn init_takes_the_data_dir() {
        let cmd: BridgeCommand =
            serde_json::from_str(r#"{"cmd":"system","args":{"type":"init","data_dir":"/data"}}"#)
                .unwrap();
        assert!(matches!(
            cmd,
            BridgeCommand::System(SystemCommand::Init(init)) if init.data_dir == "/data"
        ));
    }

    #[test]
    fn fallback_matches_the_bridge_error_contract() {
        assert_eq!(
            err_json(BridgeError::Internal {
                reason: "boom".to_string()
            }),
            r#"{"status":"error","kind":"internal","reason":"boom"}"#
        );
        assert!(SERIALIZATION_FALLBACK.contains(r#""reason""#));
    }
}

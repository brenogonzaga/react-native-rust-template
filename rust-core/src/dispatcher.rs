use crate::error::BridgeError;
use crate::handlers::math::{MathCommand, MathHandler};
use crate::handlers::system::{SystemCommand, SystemHandler};
use crate::handlers::user::{UserCommand, UserHandler};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum BridgeResponse<T> {
    Success { data: T },
    Error(BridgeError),
}

#[derive(Deserialize)]
#[serde(tag = "cmd", content = "args", rename_all = "snake_case")]
pub enum BridgeCommand {
    System(SystemCommand),
    Math(MathCommand),
    User(UserCommand),
}

pub struct BridgeDispatcher;

impl BridgeDispatcher {
    pub async fn run(command: BridgeCommand) -> String {
        let result = match command {
            BridgeCommand::System(sys_cmd) => SystemHandler::dispatch(sys_cmd).await,
            BridgeCommand::Math(math_cmd) => MathHandler::dispatch(math_cmd).await,
            BridgeCommand::User(user_cmd) => UserHandler::dispatch(user_cmd).await,
        };

        Self::wrap(result)
    }

    fn wrap<T: Serialize>(res: Result<T, BridgeError>) -> String {
        let response = match res {
            Ok(data) => BridgeResponse::Success { data },
            Err(err) => BridgeResponse::Error(err),
        };
        serde_json::to_string(&response).unwrap_or_else(|_| {
            r#"{"status":"error","kind":"internal","message":"Failed to serialize response"}"#
                .to_string()
        })
    }
}

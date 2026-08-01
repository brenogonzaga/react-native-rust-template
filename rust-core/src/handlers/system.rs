use crate::state::CORE_SERVICE;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SystemCommand {
    Ping,
    GetVersion,
}

pub struct SystemHandler;

impl SystemHandler {
    pub async fn dispatch(cmd: SystemCommand) -> Result<serde_json::Value, crate::error::BridgeError> {
        match cmd {
            SystemCommand::Ping => Ok(serde_json::json!("pong")),
            SystemCommand::GetVersion => {
                let info = CORE_SERVICE.get_version();
                Ok(serde_json::to_value(info)?)
            }
        }
    }
}

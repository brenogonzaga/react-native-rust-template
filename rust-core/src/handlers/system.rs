use crate::error::BridgeError;
use crate::state::CORE_SERVICE;
use crate::wire::Wire;
use serde::Deserialize;
use serde_json::value::RawValue;
use ts_rs::TS;

#[derive(Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SystemCommand {
    Ping,
    GetVersion,
}

pub struct SystemHandler;

impl SystemHandler {
    pub fn dispatch(cmd: SystemCommand) -> Result<Box<RawValue>, BridgeError> {
        match cmd {
            SystemCommand::Ping => Wire::encode("pong"),
            SystemCommand::GetVersion => Wire::encode(&CORE_SERVICE.get_version()),
        }
    }
}

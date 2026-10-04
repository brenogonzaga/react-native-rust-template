use crate::error::BridgeError;
use crate::state::CORE_SERVICE;
use crate::wire::Wire;
use serde::Deserialize;
use serde_json::value::RawValue;
use ts_rs::TS;

#[derive(Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserCommand {
    GetUser {
        id: String,
    },
    SaveUser {
        id: String,
        name: String,
        role: String,
    },
}

pub struct UserHandler;

impl UserHandler {
    pub fn dispatch(cmd: UserCommand) -> Result<Box<RawValue>, BridgeError> {
        match cmd {
            UserCommand::GetUser { id } => Wire::encode(&CORE_SERVICE.get_user(&id)?),
            UserCommand::SaveUser { id, name, role } => {
                Wire::encode(&CORE_SERVICE.save_user(id, name, role))
            }
        }
    }
}

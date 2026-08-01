use crate::error::BridgeError;
use crate::state::CORE_SERVICE;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserCommand {
    GetUser { id: String },
    SaveUser { id: String, name: String, role: String },
}

pub struct UserHandler;

impl UserHandler {
    pub async fn dispatch(cmd: UserCommand) -> Result<serde_json::Value, BridgeError> {
        match cmd {
            UserCommand::GetUser { id } => {
                let user = CORE_SERVICE.get_user(&id)?;
                Ok(serde_json::to_value(user)?)
            }
            UserCommand::SaveUser { id, name, role } => {
                let saved_user = CORE_SERVICE.save_user(id, name, role);
                Ok(serde_json::to_value(saved_user)?)
            }
        }
    }
}

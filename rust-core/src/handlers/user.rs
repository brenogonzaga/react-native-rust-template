use super::Command;
use crate::error::BridgeError;
use crate::state::CORE_SERVICE;
use app_core::User;
use serde::Deserialize;
use serde_json::value::RawValue;
use ts_rs::TS;

#[derive(Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserCommand {
    GetUser(GetUser),
    SaveUser(SaveUser),
}

#[derive(TS)]
#[allow(dead_code)]
pub struct UserResponses {
    pub get_user: <GetUser as Command>::Response,
    pub save_user: <SaveUser as Command>::Response,
}

impl UserCommand {
    pub fn dispatch(self) -> Result<Box<RawValue>, BridgeError> {
        match self {
            Self::GetUser(cmd) => cmd.respond(),
            Self::SaveUser(cmd) => cmd.respond(),
        }
    }
}

#[derive(Deserialize, TS)]
pub struct GetUser {
    pub id: String,
}

impl Command for GetUser {
    type Response = User;

    fn run(self) -> Result<User, BridgeError> {
        Ok(CORE_SERVICE.get_user(&self.id)?)
    }
}

#[derive(Deserialize, TS)]
pub struct SaveUser {
    pub id: String,
    pub name: String,
    pub role: String,
}

impl Command for SaveUser {
    type Response = User;

    fn run(self) -> Result<User, BridgeError> {
        Ok(CORE_SERVICE.save_user(self.id, self.name, self.role)?)
    }
}

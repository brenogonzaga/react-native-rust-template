use super::Command;
use crate::error::BridgeError;
use crate::state::CORE_SERVICE;
use app_core::AppVersion;
use serde::Deserialize;
use serde_json::value::RawValue;
use std::path::Path;
use ts_rs::TS;

#[derive(Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SystemCommand {
    Ping(Ping),
    GetVersion(GetVersion),
    Init(Init),
}

#[derive(TS)]
#[allow(dead_code)]
pub struct SystemResponses {
    pub ping: <Ping as Command>::Response,
    pub get_version: <GetVersion as Command>::Response,
    pub init: <Init as Command>::Response,
}

impl SystemCommand {
    pub fn dispatch(self) -> Result<Box<RawValue>, BridgeError> {
        match self {
            Self::Ping(cmd) => cmd.respond(),
            Self::GetVersion(cmd) => cmd.respond(),
            Self::Init(cmd) => cmd.respond(),
        }
    }
}

#[derive(Deserialize, TS)]
pub struct Ping {}

impl Command for Ping {
    type Response = String;

    fn run(self) -> Result<String, BridgeError> {
        Ok("pong".to_string())
    }
}

#[derive(Deserialize, TS)]
pub struct GetVersion {}

impl Command for GetVersion {
    type Response = AppVersion;

    fn run(self) -> Result<AppVersion, BridgeError> {
        Ok(CORE_SERVICE.get_version())
    }
}

/// Points the core at the app's writable data directory (the native module's
/// `dataDir` constant). Loads saved users from there and persists new ones.
#[derive(Deserialize, TS)]
pub struct Init {
    pub data_dir: String,
}

impl Command for Init {
    type Response = ();

    fn run(self) -> Result<(), BridgeError> {
        Ok(CORE_SERVICE.open_storage(Path::new(&self.data_dir))?)
    }
}

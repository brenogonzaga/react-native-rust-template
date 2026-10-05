pub mod math;
pub mod system;
pub mod user;

use crate::error::BridgeError;
use crate::wire::{SafeForWire, Wire};
use serde_json::value::RawValue;
use ts_rs::TS;

/// One bridge command: its args are the implementing struct, its `data` is
/// `Response`. Each handler's `*Responses` struct names these types, so the
/// response map exported to TypeScript can't drift from what `run` returns.
pub trait Command: Sized {
    type Response: SafeForWire + TS;

    fn run(self) -> Result<Self::Response, BridgeError>;

    fn respond(self) -> Result<Box<RawValue>, BridgeError> {
        Wire::encode(&self.run()?)
    }
}

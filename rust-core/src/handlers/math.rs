use crate::error::BridgeError;
use crate::safe_int::SafeInt;
use crate::state::CORE_SERVICE;
use crate::wire::Wire;
use serde::Deserialize;
use serde_json::value::RawValue;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MathCommand {
    AddNumbers { a: i64, b: i64 },
    Factorial { n: u64 },
}

pub struct MathHandler;

impl MathHandler {
    pub fn dispatch(cmd: MathCommand) -> Result<Box<RawValue>, BridgeError> {
        match cmd {
            MathCommand::AddNumbers { a, b } => {
                let sum = a
                    .checked_add(b)
                    .ok_or_else(|| BridgeError::InvalidArgument {
                        reason: format!("{a} + {b} overflows i64"),
                    })?;
                Wire::encode(&SafeInt(sum))
            }
            MathCommand::Factorial { n } => {
                let result = CORE_SERVICE.calculate_factorial(n)?;
                Wire::encode(&SafeInt(result))
            }
        }
    }
}

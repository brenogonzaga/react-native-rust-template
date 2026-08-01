use crate::error::BridgeError;
use crate::state::CORE_SERVICE;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MathCommand {
    AddNumbers { a: i64, b: i64 },
    Factorial { n: u64 },
}

pub struct MathHandler;

impl MathHandler {
    pub async fn dispatch(cmd: MathCommand) -> Result<serde_json::Value, BridgeError> {
        match cmd {
            MathCommand::AddNumbers { a, b } => {
                let sum = a + b;
                Ok(serde_json::json!(sum))
            }
            MathCommand::Factorial { n } => {
                let result = CORE_SERVICE.calculate_factorial(n)?;
                Ok(serde_json::json!(result))
            }
        }
    }
}

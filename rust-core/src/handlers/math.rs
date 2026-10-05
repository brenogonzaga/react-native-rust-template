use super::Command;
use crate::error::BridgeError;
use crate::safe_int::SafeInt;
use crate::state::CORE_SERVICE;
use serde::Deserialize;
use serde_json::value::RawValue;
use ts_rs::TS;

#[derive(Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MathCommand {
    AddNumbers(AddNumbers),
    Factorial(Factorial),
}

#[derive(TS)]
#[allow(dead_code)]
pub struct MathResponses {
    pub add_numbers: <AddNumbers as Command>::Response,
    pub factorial: <Factorial as Command>::Response,
}

impl MathCommand {
    pub fn dispatch(self) -> Result<Box<RawValue>, BridgeError> {
        match self {
            Self::AddNumbers(cmd) => cmd.respond(),
            Self::Factorial(cmd) => cmd.respond(),
        }
    }
}

#[derive(Deserialize, TS)]
pub struct AddNumbers {
    pub a: i64,
    pub b: i64,
}

impl Command for AddNumbers {
    type Response = SafeInt<i64>;

    fn run(self) -> Result<SafeInt<i64>, BridgeError> {
        let Self { a, b } = self;
        a.checked_add(b)
            .map(SafeInt)
            .ok_or_else(|| BridgeError::InvalidArgument {
                reason: format!("{a} + {b} overflows i64"),
            })
    }
}

#[derive(Deserialize, TS)]
pub struct Factorial {
    pub n: u64,
}

impl Command for Factorial {
    type Response = SafeInt<u64>;

    fn run(self) -> Result<SafeInt<u64>, BridgeError> {
        Ok(SafeInt(CORE_SERVICE.calculate_factorial(self.n)?))
    }
}

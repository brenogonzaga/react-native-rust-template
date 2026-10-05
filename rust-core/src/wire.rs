use crate::error::BridgeError;
use crate::safe_int::SafeInt;
use serde::Serialize;
use serde_json::value::RawValue;
use std::fmt;

/// Marks a type as allowed to cross the FFI boundary as JSON.
pub trait SafeForWire: Serialize {}

impl SafeForWire for () {}
impl SafeForWire for str {}
impl SafeForWire for String {}
impl<T: fmt::Display> SafeForWire for SafeInt<T> {}
impl SafeForWire for app_core::AppVersion {}
impl SafeForWire for app_core::User {}

/// The one place a handler response is actually encoded — every
/// `handlers/*.rs` dispatch arm goes through this.
pub struct Wire;

impl Wire {
    pub fn encode<T: SafeForWire + ?Sized>(value: &T) -> Result<Box<RawValue>, BridgeError> {
        Ok(serde_json::value::to_raw_value(value)?)
    }
}

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;
use ts_rs::{Config, Dummy, TS};

/// Wraps a 64-bit-or-wider integer that must cross the FFI boundary as a JSON
/// string, never a bare number — JS numbers are f64, so `JSON.parse` rounds
/// anything past 2^53 silently. `TS_RS_LARGE_INT` only changes the declared
/// TS type, not the actual encoding, so it can't fix this on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SafeInt<T>(pub T);

impl<T: fmt::Display> fmt::Display for SafeInt<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl<T: fmt::Display> Serialize for SafeInt<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.0)
    }
}

impl<'de, T> Deserialize<'de> for SafeInt<T>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse::<T>()
            .map(SafeInt)
            .map_err(serde::de::Error::custom)
    }
}

impl<T> TS for SafeInt<T> {
    type WithoutGenerics = SafeInt<Dummy>;
    type OptionInnerType = Self;

    fn name(_: &Config) -> String {
        String::from("string")
    }

    fn inline(cfg: &Config) -> String {
        <Self as TS>::name(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_as_a_json_string_past_f64_safe_range() {
        let huge = SafeInt(u64::MAX - 5);
        let json = serde_json::to_string(&huge).unwrap();
        assert_eq!(json, format!(r#""{}""#, u64::MAX - 5));
    }

    #[test]
    fn round_trips_through_deserialize() {
        let original = SafeInt(9_007_199_254_740_993i64); // 2^53 + 1
        let json = serde_json::to_string(&original).unwrap();
        let back: SafeInt<i64> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn ts_type_is_string() {
        assert_eq!(SafeInt::<u64>::name(&Config::default()), "string");
    }
}

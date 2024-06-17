use std::str::FromStr;

/// Represents a Boolean value
/// with the following representations in Line Protocol
///
/// Accepted Syntax for True value: `t`, `T`, `true`, `True`, or `TRUE`
///
/// Accepted Syntax for False value: `f`, `F`, `false`, `False`, or `FALSE`
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Deref,
    derive_more::Into,
    derive_more::From,
    derive_more::Display,
)]
pub struct Boolean(bool);

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse value as Boolean")]
pub struct BooleanParseError;

impl FromStr for Boolean {
    type Err = BooleanParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "t" | "T" | "true" | "True" | "TRUE" => return Ok(Boolean(true)),
            "f" | "F" | "false" | "False" | "FALSE" => return Ok(Boolean(false)),
            _ => Err(BooleanParseError),
        }
    }
}

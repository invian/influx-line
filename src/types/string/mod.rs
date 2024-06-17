mod formatter;
mod key;
mod measurement;
mod parser;

pub use self::key::KeyName;
pub use self::measurement::MeasurementName;
pub use self::parser::NameParseError;

#[derive(Debug, thiserror::Error)]
#[error("Name does not abide by naming restrictions")]
pub struct NameRestrictionError;

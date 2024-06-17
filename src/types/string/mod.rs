mod formatter;
mod key;
mod measurement;
mod parser;
mod quoted;

pub use self::key::KeyName;
pub use self::measurement::MeasurementName;
pub use self::parser::ParseError;
pub use self::quoted::QuotedString;

#[derive(Debug, thiserror::Error)]
#[error("Name does not abide by naming restrictions")]
pub struct NameRestrictionError;

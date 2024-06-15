/// A measurement, tag, or field name.
///
/// Subject to [Naming restrictions](
/// https://docs.influxdata.com/influxdb/v2/reference/syntax/line-protocol/#naming-restrictions
/// ).
///
/// # Examples
///
/// ```rust
/// use influx_line::*;
///
/// let measurement = InfluxName::try_from(String::from("measurement")).unwrap();
/// assert_eq!(measurement, "measurement");
///
/// let malformed_name = InfluxName::try_from("_bad").unwrap_err();
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Into,
    derive_more::Deref,
    derive_more::Index,
)]
pub struct InfluxName(String);

#[derive(Debug, thiserror::Error)]
#[error("Invalid name: {0}")]
pub struct MalformedNameError(String);

impl TryFrom<String> for InfluxName {
    type Error = MalformedNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() || value.starts_with('_') {
            return Err(MalformedNameError(value));
        }

        Ok(Self(value))
    }
}

impl TryFrom<&str> for InfluxName {
    type Error = MalformedNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(String::from(value))
    }
}

impl AsRef<str> for InfluxName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl PartialEq<str> for InfluxName {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

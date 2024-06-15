use chrono::{DateTime, Utc};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::From, derive_more::Into,
)]
pub struct Timestamp(i64);

impl From<Timestamp> for DateTime<Utc> {
    fn from(value: Timestamp) -> Self {
        DateTime::from_timestamp_nanos(value.into()).to_utc()
    }
}

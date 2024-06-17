use std::str::FromStr;

use chrono::{DateTime, Utc};

/// Represents a Timestamp (in nanoseconds) at the end of the Line Protocol.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::From,
    derive_more::Into,
    derive_more::Display,
)]
#[from(types(u8, u16, u32, i8, i16, i32))]
pub struct Timestamp(i64);

#[derive(Debug, thiserror::Error)]
#[error("`{0}` is not a valid timestamp: {1}")]
pub struct TimestampParseError(String, std::num::ParseIntError);

impl From<Timestamp> for DateTime<Utc> {
    fn from(value: Timestamp) -> Self {
        DateTime::from_timestamp_nanos(value.into()).to_utc()
    }
}

impl FromStr for Timestamp {
    type Err = TimestampParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let timestamp = s
            .parse::<i64>()
            .map_err(|parse_int_error| TimestampParseError(s.into(), parse_int_error))?;
        Ok(Self(timestamp))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::Timestamp;

    #[rstest::rstest]
    #[case::big_timestamp("1556813561098000000", 1556813561098000000)]
    #[case::negative_timestamp("-100500", -100500)]
    fn successful_parsing(#[case] input: &str, #[case] expected_value: i64) {
        let expected_timestamp = Timestamp::from(expected_value);

        let actual_timestamp = Timestamp::from_str(input).expect("Must parse here");

        assert_eq!(expected_timestamp, actual_timestamp);
    }

    #[rstest::rstest]
    #[case::empty("")]
    #[case::gibberish("abcdefg")]
    #[case::influx_int_as_timestamp("123i")]
    #[case::influx_float_as_timestamp("128.0")]
    fn parse_error(#[case] input: &str) {
        let _parse_error = Timestamp::from_str(input).unwrap_err();
    }

    #[rstest::rstest]
    #[case(1556813561098000000, "1556813561098000000")]
    fn display(#[case] value: i64, #[case] expected_string: &str) {
        let actual_string = Timestamp::from(value).to_string();

        assert_eq!(expected_string, actual_string);
    }
}

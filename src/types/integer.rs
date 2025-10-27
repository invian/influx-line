use std::str::FromStr;

use crate::InfluxLineError;

/// Represents an Integer value with custom format.
///
/// Specifically, in Line Protocol, Integer values must end with `i` as follows
///
/// - `128i`
/// - `0i`
/// - `-99999i`
///
/// Otherwise, the value will be treated as Float.
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
#[from(i8, i16, i32, i64)]
#[display("{}i", _0)]
pub struct InfluxInteger(i64);

/// Represents an Unsigned Integer value with custom format.
///
/// Specifically, in Line Protocol, Unsigned Integer values must end with `u` as follows
///
/// - `128u`
/// - `0u`
///
/// Otherwise, the value will be treated as Float.
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
#[from(u8, u16, u32, u64)]
#[display("{}u", _0)]
pub struct InfluxUInteger(u64);

impl FromStr for InfluxInteger {
    type Err = InfluxLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((int_slice, empty)) = s.split_once('i') else {
            return Err(InfluxLineError::IntegerNotParsed);
        };
        if !empty.is_empty() {
            return Err(InfluxLineError::IntegerNotParsed);
        }

        let integer = int_slice
            .parse::<i64>()
            .map_err(|_| InfluxLineError::IntegerNotParsed)?;

        Ok(Self(integer))
    }
}

impl FromStr for InfluxUInteger {
    type Err = InfluxLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((uint_slice, empty)) = s.split_once('u') else {
            return Err(InfluxLineError::UIntegerNotParsed);
        };
        if !empty.is_empty() {
            return Err(InfluxLineError::UIntegerNotParsed);
        }

        let uinteger = uint_slice
            .parse::<u64>()
            .map_err(|_| InfluxLineError::UIntegerNotParsed)?;

        Ok(Self(uinteger))
    }
}

impl TryFrom<InfluxInteger> for i32 {
    type Error = InfluxLineError;

    fn try_from(value: InfluxInteger) -> Result<Self, Self::Error> {
        value
            .0
            .try_into()
            .map_err(|_| InfluxLineError::TypeConversion)
    }
}

impl TryFrom<InfluxInteger> for i16 {
    type Error = InfluxLineError;

    fn try_from(value: InfluxInteger) -> Result<Self, Self::Error> {
        value
            .0
            .try_into()
            .map_err(|_| InfluxLineError::TypeConversion)
    }
}

impl TryFrom<InfluxInteger> for i8 {
    type Error = InfluxLineError;

    fn try_from(value: InfluxInteger) -> Result<Self, Self::Error> {
        value
            .0
            .try_into()
            .map_err(|_| InfluxLineError::TypeConversion)
    }
}

impl TryFrom<InfluxUInteger> for u32 {
    type Error = InfluxLineError;

    fn try_from(value: InfluxUInteger) -> Result<Self, Self::Error> {
        value
            .0
            .try_into()
            .map_err(|_| InfluxLineError::TypeConversion)
    }
}

impl TryFrom<InfluxUInteger> for u16 {
    type Error = InfluxLineError;

    fn try_from(value: InfluxUInteger) -> Result<Self, Self::Error> {
        value
            .0
            .try_into()
            .map_err(|_| InfluxLineError::TypeConversion)
    }
}

impl TryFrom<InfluxUInteger> for u8 {
    type Error = InfluxLineError;

    fn try_from(value: InfluxUInteger) -> Result<Self, Self::Error> {
        value
            .0
            .try_into()
            .map_err(|_| InfluxLineError::TypeConversion)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{InfluxInteger, InfluxUInteger};

    #[rstest::rstest]
    #[case("123i", 123.into())]
    #[case("0i", 0.into())]
    #[case("-25565i", (-25565).into())]
    fn successful_int_parsing(#[case] input: &str, #[case] expected_integer: InfluxInteger) {
        let actual_integer = InfluxInteger::from_str(input).expect("Must parse here");

        assert_eq!(expected_integer, actual_integer)
    }

    #[rstest::rstest]
    #[case::uint("123u")]
    #[case::no_suffix_means_float("0")]
    #[case::actual_float("128.0")]
    #[case::empty("")]
    #[case::gibberish("randomi")]
    #[case::spaces("123 01i")]
    fn int_parse_error(#[case] input: &str) {
        let _parse_error = InfluxInteger::from_str(input).expect_err("Must return parse error");
    }

    #[rstest::rstest]
    #[case("123u", (123 as u32).into())]
    #[case("0u", (0 as u32).into())]
    fn successful_uint_parsing(#[case] input: &str, #[case] expected_integer: InfluxUInteger) {
        let actual_integer = InfluxUInteger::from_str(input).expect("Must parse here");

        assert_eq!(expected_integer, actual_integer)
    }

    #[rstest::rstest]
    #[case::positive_int("25565i")]
    #[case::negative_int("-25565i")]
    #[case::negative_uint("-25565u")]
    #[case::no_suffix_means_float("0")]
    #[case::actual_float("128.0")]
    #[case::empty("")]
    #[case::gibberish("randomu")]
    #[case::spaces("123 01u")]
    fn uint_parse_error(#[case] input: &str) {
        let _parse_error = InfluxUInteger::from_str(input).expect_err("Must return parse error");
    }
}

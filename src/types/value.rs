use std::str::FromStr;

use crate::{Boolean, InfluxInteger, InfluxLineError, InfluxUInteger, QuotedString};

#[derive(
    Debug, Clone, PartialEq, derive_more::From, derive_more::TryInto, derive_more::Display,
)]
pub enum InfluxValue {
    #[display(fmt = "{:?}", _0)]
    #[from(types(f32))]
    Float(f64),
    #[from(types(i8, i16, i32, i64))]
    Integer(InfluxInteger),
    #[from(types(u8, u16, u32, u64))]
    UInteger(InfluxUInteger),
    #[from(types(bool))]
    Boolean(Boolean),
    #[from]
    String(QuotedString),
}

impl From<&str> for InfluxValue {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl FromStr for InfluxValue {
    type Err = InfluxLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(float) = s.parse::<f64>() {
            return Ok(float.into());
        }

        if let Ok(integer) = s.parse::<InfluxInteger>() {
            return Ok(integer.into());
        }

        if let Ok(uinteger) = s.parse::<InfluxUInteger>() {
            return Ok(uinteger.into());
        }

        if let Ok(boolean) = s.parse::<Boolean>() {
            return Ok(boolean.into());
        }

        if let Ok(quoted_string) = s.parse::<QuotedString>() {
            return Ok(quoted_string.into());
        }

        Err(InfluxLineError::BadValue)
    }
}

impl TryFrom<InfluxValue> for f32 {
    type Error = InfluxLineError;

    fn try_from(value: InfluxValue) -> Result<Self, Self::Error> {
        match value {
            InfluxValue::Float(inner) => Ok(inner as f32),
            _ => Err(InfluxLineError::TypeConversion),
        }
    }
}

impl TryFrom<InfluxValue> for i8 {
    type Error = InfluxLineError;

    fn try_from(value: InfluxValue) -> Result<Self, Self::Error> {
        match value {
            InfluxValue::Integer(inner) => inner.try_into(),
            _ => Err(InfluxLineError::TypeConversion),
        }
    }
}

impl TryFrom<InfluxValue> for i16 {
    type Error = InfluxLineError;

    fn try_from(value: InfluxValue) -> Result<Self, Self::Error> {
        match value {
            InfluxValue::Integer(inner) => inner.try_into(),
            _ => Err(InfluxLineError::TypeConversion),
        }
    }
}

impl TryFrom<InfluxValue> for i32 {
    type Error = InfluxLineError;

    fn try_from(value: InfluxValue) -> Result<Self, Self::Error> {
        match value {
            InfluxValue::Integer(inner) => inner.try_into(),
            _ => Err(InfluxLineError::TypeConversion),
        }
    }
}

impl TryFrom<InfluxValue> for i64 {
    type Error = InfluxLineError;

    fn try_from(value: InfluxValue) -> Result<Self, Self::Error> {
        match value {
            InfluxValue::Integer(inner) => Ok(inner.into()),
            _ => Err(InfluxLineError::TypeConversion),
        }
    }
}

impl TryFrom<InfluxValue> for u8 {
    type Error = InfluxLineError;

    fn try_from(value: InfluxValue) -> Result<Self, Self::Error> {
        match value {
            InfluxValue::UInteger(inner) => inner.try_into(),
            _ => Err(InfluxLineError::TypeConversion),
        }
    }
}

impl TryFrom<InfluxValue> for u16 {
    type Error = InfluxLineError;

    fn try_from(value: InfluxValue) -> Result<Self, Self::Error> {
        match value {
            InfluxValue::UInteger(inner) => inner.try_into(),
            _ => Err(InfluxLineError::TypeConversion),
        }
    }
}

impl TryFrom<InfluxValue> for u32 {
    type Error = InfluxLineError;

    fn try_from(value: InfluxValue) -> Result<Self, Self::Error> {
        match value {
            InfluxValue::UInteger(inner) => inner.try_into(),
            _ => Err(InfluxLineError::TypeConversion),
        }
    }
}

impl TryFrom<InfluxValue> for u64 {
    type Error = InfluxLineError;

    fn try_from(value: InfluxValue) -> Result<Self, Self::Error> {
        match value {
            InfluxValue::UInteger(inner) => Ok(inner.into()),
            _ => Err(InfluxLineError::TypeConversion),
        }
    }
}

impl TryFrom<InfluxValue> for bool {
    type Error = InfluxLineError;

    fn try_from(value: InfluxValue) -> Result<Self, Self::Error> {
        match value {
            InfluxValue::Boolean(inner) => Ok(inner.into()),
            _ => Err(InfluxLineError::TypeConversion),
        }
    }
}

impl TryFrom<InfluxValue> for String {
    type Error = InfluxLineError;

    fn try_from(value: InfluxValue) -> Result<Self, Self::Error> {
        match value {
            InfluxValue::String(inner) => Ok(inner.into()),
            _ => Err(InfluxLineError::TypeConversion),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::InfluxValue;

    #[rstest::rstest]
    #[case::sane_float("12.33", InfluxValue::Float(12.33))]
    #[case::float_without_dots("1", InfluxValue::Float(1.0))]
    #[case::negative_with_scientific_stuff("-1.234456e+78", InfluxValue::Float(-1.234456e+78))]
    #[case::positive_int("125i", InfluxValue::Integer(125.into()))]
    #[case::negative_int("-25565i", InfluxValue::Integer((-25565).into()))]
    #[case::uint("999999999u", InfluxValue::UInteger((999999999 as u32).into()))]
    #[case::le_true("true", InfluxValue::Boolean(true.into()))]
    #[case::le_false("FALSE", InfluxValue::Boolean(false.into()))]
    #[case::string("\"Dunno what to say\"", InfluxValue::String("Dunno what to say".into()))]
    fn successful_parsing(#[case] input: &str, #[case] expected_value: InfluxValue) {
        let actual_value = InfluxValue::from_str(input).expect("Must parse here");

        assert_eq!(expected_value, actual_value);
    }

    #[rstest::rstest]
    #[case::sane_float(17.0, "17.0")]
    #[case::float_strange(25 as f32, "25.0")]
    #[case::int(15, "15i")]
    #[case::uint(0 as u32, "0u")]
    #[case::le_true(true, "true")]
    #[case::le_false(false, "false")]
    #[case::string("eat \"this\" hehe", "\"eat \\\"this\\\" hehe\"")]
    fn display<T>(#[case] value: T, #[case] expected_string: &str)
    where
        InfluxValue: From<T>,
    {
        let actual_string = InfluxValue::from(value).to_string();

        assert_eq!(expected_string, actual_string);
    }
}

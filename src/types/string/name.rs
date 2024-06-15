use std::str::FromStr;

use super::parser::LinearParser;

/// A measurement name with special restrictions on parsing and formatting stage.
///
/// Subject to [Naming restrictions](
/// https://docs.influxdata.com/influxdb/v2/reference/syntax/line-protocol/#naming-restrictions
/// ).
///
/// # Examples
///
/// ## Creating instances
///
/// [`TryFrom`] implementation allows inserting human readable values.
///
/// ```rust
/// use influx_line::*;
///
/// let measurement = MeasurementName::try_from(String::from("measurement")).unwrap();
/// let raw_chicken = MeasurementName::try_from("raw chicken").unwrap();
///
/// assert_eq!(measurement, "measurement");
/// assert_eq!(raw_chicken, "raw chicken");
/// ```
///
/// ## Naming restrictions
///
/// > The `_` namespace is reserved for InfluxDB system use.
///
/// ```rust
/// use influx_line::*;
///
/// let _error = MeasurementName::try_from("_bad").unwrap_err();
/// ```
///
/// ## Parsing from Line Protocol representation
///
/// [`FromStr`] implementation is made as a part of Line Protocol parsing.
///
/// ```rust
/// use influx_line::*;
/// use std::str::FromStr;
///
/// let measurement = MeasurementName::from_str(r#"escaped\ name"#).unwrap();
/// assert_eq!(measurement, "escaped name");
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
pub struct MeasurementName(String);

#[derive(Debug, thiserror::Error)]
#[error("Invalid name: {0}")]
pub struct MalformedNameError(String);

#[derive(Debug, thiserror::Error)]
pub enum NameParseError {
    #[error("Failed to parse name: {0}")]
    Failed(String),
    #[error("Failed to process character at position `{1}`: {0}")]
    BadCharacter(String, usize),
    #[error(transparent)]
    Malformed(#[from] MalformedNameError),
}

impl MeasurementName {
    const SPECIAL_CHARACTERS: [char; 3] = [' ', '=', ','];
    const ESCAPE_CHARACTER: char = '\\';

    #[cfg(test)]
    fn unchecked<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self(name.into())
    }
}

impl TryFrom<String> for MeasurementName {
    type Error = MalformedNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() || value.starts_with('_') {
            return Err(MalformedNameError(value));
        }

        Ok(Self(value))
    }
}

impl TryFrom<&str> for MeasurementName {
    type Error = MalformedNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(String::from(value))
    }
}

impl AsRef<str> for MeasurementName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl PartialEq<str> for MeasurementName {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for MeasurementName {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for MeasurementName {
    fn eq(&self, other: &String) -> bool {
        self.0 == other.as_str()
    }
}

impl FromStr for MeasurementName {
    type Err = NameParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser =
            LinearParser::new(Self::SPECIAL_CHARACTERS.to_vec(), Self::ESCAPE_CHARACTER);

        for (index, character) in s.chars().enumerate() {
            let is_processed = parser.process_char(character);
            if !is_processed {
                return Err(NameParseError::BadCharacter(s.into(), index));
            }
        }

        let parsed = parser.extract().ok_or(NameParseError::Failed(s.into()))?;
        let name = MeasurementName::try_from(parsed)?;
        Ok(name)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::MeasurementName;

    #[rstest::rstest]
    #[case::no_special_characters(r#"amogus"#, MeasurementName::unchecked("amogus"))]
    #[case::unescaped_quote(r#"stupid"quote"#, MeasurementName::unchecked(r#"stupid"quote"#))]
    #[case::escaped_space(r#"hello\ man"#, MeasurementName::unchecked("hello man"))]
    #[case::escaped_comma(
        r#"milk\,bread\,butter"#,
        MeasurementName::unchecked("milk,bread,butter")
    )]
    #[case::escaped_equals(r#"a\=b"#, MeasurementName::unchecked("a=b"))]
    #[case::slashes_1_1(r#"a\a"#, MeasurementName::unchecked(r#"a\a"#))]
    #[case::slashes_2_1(r#"a\\a"#, MeasurementName::unchecked(r#"a\a"#))]
    #[case::slashes_3_2(r#"a\\\a"#, MeasurementName::unchecked(r#"a\\a"#))]
    #[case::slashes_4_2(r#"a\\\\a"#, MeasurementName::unchecked(r#"a\\a"#))]
    #[case::slashes_5_3(r#"a\\\\\a"#, MeasurementName::unchecked(r#"a\\\a"#))]
    #[case::slashes_6_3(r#"a\\\\\\a"#, MeasurementName::unchecked(r#"a\\\a"#))]
    #[case::double_trailing_slash(r#"haha\\"#, MeasurementName::unchecked(r#"haha\"#))]
    #[case::everything(
        r#"day\ when\ f(x\,\ y)\ \=\ 10"#,
        MeasurementName::unchecked("day when f(x, y) = 10")
    )]
    #[case::unicode(r#"💀\ dead\ man\ 💀"#, MeasurementName::unchecked("💀 dead man 💀"))]
    fn successful_parsing(#[case] escaped_input: &str, #[case] expected_name: MeasurementName) {
        let actual_name = MeasurementName::from_str(&escaped_input).expect("Must parse here");

        assert_eq!(expected_name, actual_name);
    }

    #[rstest::rstest]
    #[case::empty("")]
    #[case::unescaped_space(r#"hello kitty"#)]
    #[case::unescaped_comma(r#"you,me,together..."#)]
    #[case::unescaped_equals(r#"1+1=10"#)]
    #[case::trailing_slash(r#"we\ are\ number\ one\"#)]
    #[case::starts_with_underscore(r#"_reserved"#)]
    fn parsing_fails(#[case] escaped_input: &str) {
        let _parse_error = MeasurementName::from_str(escaped_input).expect_err("Must return error");
    }
}

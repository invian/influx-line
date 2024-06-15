use std::fmt::Display;
use std::str::FromStr;

use crate::types::string::formatter::LinearFormatter;

use super::{parser::LinearParser, NameParseError, NameRestrictionError};

/// A measurement name with special restrictions on parsing and formatting stage.
///
/// Subject to [Naming restrictions](
/// https://docs.influxdata.com/influxdb/v2/reference/syntax/line-protocol/#naming-restrictions
/// ).
///
/// # Examples
///
/// ## Line Protocol Representation
///
/// - [`FromStr`] parses from the Line Protocol.
/// - [`Display`] trait formats for Line Protocol.
///
/// ```rust
/// use std::fmt::Display;
/// use std::str::FromStr;
/// use influx_line::*;
///
/// let line_protocol_repr = r#"escaped\ name\,\ has\ commas"#;
/// let measurement = MeasurementName::from_str(line_protocol_repr).unwrap();
///
/// assert_eq!(measurement, "escaped name, has commas");
/// assert_eq!(line_protocol_repr, measurement.to_string());
/// ```
///
/// ## Creating instances
///
/// [`TryFrom`] implementation allows inserting human readable values
/// (i.e., no need to parse escaped sequences).
///
/// [`AsRef<str>`] makes the type almost equivalent to built-in strings.
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
/// "The `_` namespace is reserved for InfluxDB system use".
///
/// ```rust
/// use influx_line::*;
///
/// let _error = MeasurementName::try_from("_bad").unwrap_err();
/// ```
///
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

impl MeasurementName {
    const SPECIAL_CHARACTERS: [char; 2] = [' ', ','];
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
    type Error = NameRestrictionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() || value.starts_with('_') {
            return Err(NameRestrictionError);
        }

        Ok(Self(value))
    }
}

impl TryFrom<&str> for MeasurementName {
    type Error = NameRestrictionError;

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

        s.chars()
            .try_for_each(|character| parser.process_char(character))?;

        let name = MeasurementName::try_from(parser.extract()?)?;
        Ok(name)
    }
}

impl Display for MeasurementName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatter = LinearFormatter::new(&Self::SPECIAL_CHARACTERS, &Self::ESCAPE_CHARACTER);
        write!(f, "{}", formatter.chars(&self).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::MeasurementName;

    #[rstest::rstest]
    #[case::no_special_characters(r#"amogus"#, MeasurementName::unchecked("amogus"))]
    #[case::unescaped_equals(r#"1+1=10"#, MeasurementName::unchecked(r#"1+1=10"#))]
    #[case::escaped_equals(r#"a\=b"#, MeasurementName::unchecked(r#"a\=b"#))]
    #[case::unescaped_quote(r#"stupid"quote"#, MeasurementName::unchecked(r#"stupid"quote"#))]
    #[case::escaped_space(r#"hello\ man"#, MeasurementName::unchecked("hello man"))]
    #[case::escaped_comma(
        r#"milk\,bread\,butter"#,
        MeasurementName::unchecked("milk,bread,butter")
    )]
    #[case::slashes_1_1(r#"a\a"#, MeasurementName::unchecked(r#"a\a"#))]
    #[case::slashes_2_1(r#"a\\a"#, MeasurementName::unchecked(r#"a\a"#))]
    #[case::slashes_3_2(r#"a\\\a"#, MeasurementName::unchecked(r#"a\\a"#))]
    #[case::slashes_4_2(r#"a\\\\a"#, MeasurementName::unchecked(r#"a\\a"#))]
    #[case::slashes_5_3(r#"a\\\\\a"#, MeasurementName::unchecked(r#"a\\\a"#))]
    #[case::slashes_6_3(r#"a\\\\\\a"#, MeasurementName::unchecked(r#"a\\\a"#))]
    #[case::double_trailing_slash(r#"haha\\"#, MeasurementName::unchecked(r#"haha\"#))]
    #[case::everything(
        r#"day\ when\ f(x\,\ y)\ =\ 10"#,
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
    #[case::trailing_slash(r#"we\ are\ number\ one\"#)]
    #[case::starts_with_underscore(r#"_reserved"#)]
    fn parsing_fails(#[case] escaped_input: &str) {
        let _parse_error = MeasurementName::from_str(escaped_input).expect_err("Must return error");
    }

    #[rstest::rstest]
    #[case::with_space(MeasurementName::unchecked(r#"john cena"#), r#"john\ cena"#)]
    #[case::with_comma(MeasurementName::unchecked(r#"you,me"#), r#"you\,me"#)]
    #[case::silly_escapes_combination(MeasurementName::unchecked(r#"a\ b"#), r#"a\\ b"#)]
    fn display(#[case] name: MeasurementName, #[case] expected_string: &str) {
        let actual_string = name.to_string();

        assert_eq!(expected_string, actual_string);
    }
}

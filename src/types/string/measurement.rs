use std::fmt::Display;
use std::str::FromStr;

use crate::types::string::formatter::LinearFormatter;

use super::{parser::LinearParser, NameParseError, NameRestrictionError};

/// Represents a measurement name,
/// and takes into account its [Naming restrictions](
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
/// assert_eq!(measurement.as_str(), "escaped name, has commas");
/// assert_eq!(line_protocol_repr, measurement.to_string());
/// ```
///
/// ## Creating instances
///
/// Two methods are available.
/// Both of them accept human readable strings
/// (i.e., no need to escape characters).
///
/// - A dedicated constructor - [`Self::new`].
/// - A polymorphic [`TryFrom`] implementation.
///
/// ```rust
/// use influx_line::*;
///
/// let chicken: String = "raw chicken".into();
/// let raw_chicken = MeasurementName::new(chicken).unwrap();
///
/// let measurement = MeasurementName::new("measurement").unwrap();
///
/// assert_eq!(raw_chicken.as_str(), "raw chicken");
/// assert_eq!(measurement.as_str(), "measurement");
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
    const SPECIAL_CHARACTERS: [char; 2] = [',', ' '];
    const ESCAPE_CHARACTER: char = '\\';

    pub fn new<S>(name: S) -> Result<Self, NameRestrictionError>
    where
        S: AsRef<str> + Into<String>,
    {
        if name.as_ref().is_empty() || name.as_ref().starts_with('_') {
            return Err(NameRestrictionError);
        }

        Ok(Self(name.into()))
    }
}

impl TryFrom<String> for MeasurementName {
    type Error = NameRestrictionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for MeasurementName {
    type Error = NameRestrictionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl AsRef<str> for MeasurementName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
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
        write!(f, "{}", formatter.chars(self).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::MeasurementName;

    #[rstest::rstest]
    #[case::no_special_characters(r#"amogus"#, "amogus")]
    #[case::unescaped_equals(r#"1+1=10"#, r#"1+1=10"#)]
    #[case::escaped_equals(r#"a\=b"#, r#"a\=b"#)]
    #[case::unescaped_quote(r#"stupid"quote"#, r#"stupid"quote"#)]
    #[case::escaped_space(r#"hello\ man"#, "hello man")]
    #[case::escaped_comma(r#"milk\,bread\,butter"#, "milk,bread,butter")]
    #[case::slashes_1_1(r#"a\a"#, r#"a\a"#)]
    #[case::slashes_2_1(r#"a\\a"#, r#"a\a"#)]
    #[case::slashes_3_2(r#"a\\\a"#, r#"a\\a"#)]
    #[case::slashes_4_2(r#"a\\\\a"#, r#"a\\a"#)]
    #[case::slashes_5_3(r#"a\\\\\a"#, r#"a\\\a"#)]
    #[case::slashes_6_3(r#"a\\\\\\a"#, r#"a\\\a"#)]
    #[case::double_trailing_slash(r#"haha\\"#, r#"haha\"#)]
    #[case::everything(r#"day\ when\ f(x\,\ y)\ =\ 10"#, "day when f(x, y) = 10")]
    #[case::unicode(r#"💀\ dead\ man\ 💀"#, "💀 dead man 💀")]
    fn successful_parsing(#[case] escaped_input: &str, #[case] expected_raw: &str) {
        let expected_name = MeasurementName::new(expected_raw).expect("Must be a valid name");

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
    #[case::with_space(r#"john cena"#, r#"john\ cena"#)]
    #[case::with_comma(r#"you,me"#, r#"you\,me"#)]
    #[case::silly_escapes_combination(r#"a\ b"#, r#"a\\ b"#)]
    fn display(#[case] input: &str, #[case] expected_string: &str) {
        let name = MeasurementName::new(input).expect("Must be a valid name");

        let actual_string = name.to_string();

        assert_eq!(expected_string, actual_string);
    }
}

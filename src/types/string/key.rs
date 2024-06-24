use std::fmt::Display;
use std::str::FromStr;

use crate::types::string::formatter::LinearFormatter;
use crate::InfluxLineError;

use super::parser::{LinearParser, StrayEscapes};

/// Represents a Tag or Field name,
/// and takes into account its [Naming restrictions](
/// https://docs.influxdata.com/influxdb/v2/reference/syntax/line-protocol/#naming-restrictions
/// ).
///
/// Automatically escapes special symbols on parsing and formatting.
/// Special symbols are comma `','`, equals sign `'='` , and space `' '`.
///
/// # Examples
///
/// ## Line Protocol Representation
///
/// - [`FromStr`] parses from the Line Protocol.
/// - [`Display`] trait formats for Line Protocol.
///
/// If you want to display an unescaped name instead, use [`AsRef<str>`].
///
/// ```rust
/// use std::fmt::Display;
/// use std::str::FromStr;
/// use influx_line::*;
///
/// let line_protocol_repr = r#"key\ a\=b\,\ very\ special"#;
/// let tag_name = KeyName::from_str(line_protocol_repr).unwrap();
///
/// assert_eq!(tag_name.as_str(), "key a=b, very special");
/// assert_eq!(line_protocol_repr, tag_name.to_string());
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
/// [`AsRef<str>`] accesses the unescaped string.
///
/// ```rust
/// use influx_line::*;
///
/// let cutie = KeyName::new("unescaped =, are okay").unwrap();
///
/// assert_eq!(cutie.as_str(), "unescaped =, are okay");
/// assert_eq!(cutie.as_ref(), "unescaped =, are okay");
/// ```
///
/// ## Naming restrictions
///
/// "The `_` namespace is reserved for InfluxDB system use".
///
/// ```rust
/// use influx_line::*;
///
/// let _error = KeyName::try_from("_bad").unwrap_err();
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
pub struct KeyName(String);

impl KeyName {
    const SPECIAL_CHARACTERS: [char; 3] = [',', '=', ' '];
    const ESCAPE_CHARACTER: char = '\\';

    pub fn new<S>(name: S) -> Result<Self, InfluxLineError>
    where
        S: AsRef<str> + Into<String>,
    {
        if name.as_ref().is_empty() || name.as_ref().starts_with('_') {
            return Err(InfluxLineError::NameRestriction);
        }

        Ok(Self(name.into()))
    }
}

impl TryFrom<String> for KeyName {
    type Error = InfluxLineError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for KeyName {
    type Error = InfluxLineError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl AsRef<str> for KeyName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl FromStr for KeyName {
    type Err = InfluxLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser = LinearParser::new(
            &Self::SPECIAL_CHARACTERS,
            &Self::ESCAPE_CHARACTER,
            StrayEscapes::Allow,
        );

        s.chars()
            .try_for_each(|character| parser.process_char(character))?;

        let name = KeyName::new(parser.extract()?)?;
        Ok(name)
    }
}

impl Display for KeyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatter = LinearFormatter::new(&Self::SPECIAL_CHARACTERS, &Self::ESCAPE_CHARACTER);
        write!(f, "{}", formatter.chars(self).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::KeyName;

    #[rstest::rstest]
    #[case::no_special_characters(r#"amogus"#, "amogus")]
    #[case::unescaped_quote(r#"stupid"quote"#, r#"stupid"quote"#)]
    #[case::escaped_equals(r#"a\=b"#, r#"a=b"#)]
    #[case::escaped_space(r#"hello\ man"#, "hello man")]
    #[case::escaped_comma(r#"milk\,bread\,butter"#, "milk,bread,butter")]
    #[case::slashes_1_1(r#"a\a"#, r#"a\a"#)]
    #[case::slashes_2_1(r#"a\\a"#, r#"a\a"#)]
    #[case::slashes_3_2(r#"a\\\a"#, r#"a\\a"#)]
    #[case::slashes_4_2(r#"a\\\\a"#, r#"a\\a"#)]
    #[case::slashes_5_3(r#"a\\\\\a"#, r#"a\\\a"#)]
    #[case::slashes_6_3(r#"a\\\\\\a"#, r#"a\\\a"#)]
    #[case::double_trailing_slash(r#"haha\\"#, r#"haha\"#)]
    #[case::everything(r#"day\ when\ f(x\,\ y)\ \=\ 10"#, "day when f(x, y) = 10")]
    #[case::unicode(r#"💀\ dead\ man\ 💀"#, "💀 dead man 💀")]
    fn successful_parsing(#[case] escaped_input: &str, #[case] expected_raw: &str) {
        let expected_name = KeyName::new(expected_raw).expect("Must be a valid name");

        let actual_name = KeyName::from_str(&escaped_input).expect("Must parse here");

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
        let _parse_error = KeyName::from_str(escaped_input).expect_err("Must return error");
    }

    #[rstest::rstest]
    #[case::with_space(r#"john cena"#, r#"john\ cena"#)]
    #[case::with_comma(r#"you,me"#, r#"you\,me"#)]
    #[case::silly_escapes_combination(r#"a\ b"#, r#"a\\ b"#)]
    fn display(#[case] input: &str, #[case] expected_string: &str) {
        let name = KeyName::new(input).expect("Must be a valid name");

        let actual_string = name.to_string();

        assert_eq!(expected_string, actual_string);
    }
}

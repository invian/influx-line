use std::fmt::Display;
use std::str::FromStr;

use crate::{line::InfluxLineError, types::string::formatter::LinearFormatter};

use super::parser::{LinearParser, StrayEscapes};

/// Represents a String Field value in Line Protocol.
///
/// String values have the following limitations:
///
/// - They must be quoted: `field="String"`
/// - Special characters (backslash and double quote) must be escaped:
///   `"Special \" characters \\ escaped"`
///
/// Working with Quoted Strings does not require any special magic.
/// [`std::str::FromStr`] and [`std::fmt::Display`] trait implementations
/// parse and format the string automatically, handling escape symbols and double quotes.
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
pub struct QuotedString(String);

impl QuotedString {
    const SPECIAL_CHARACTERS: [char; 2] = ['"', '\\'];
    const ESCAPE_CHARACTER: char = '\\';

    /// Creates a Quoted String from a raw value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::str::FromStr;
    /// use influx_line::*;
    ///
    /// let manual = QuotedString::new("no escape");
    /// let parsed = QuotedString::from_str("\"no escape\"").unwrap();
    ///
    /// assert_eq!(manual, parsed);
    /// ```
    pub fn new<S>(value: S) -> Self
    where
        S: Into<String>,
    {
        Self(value.into())
    }
}

impl From<String> for QuotedString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for QuotedString {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl AsRef<str> for QuotedString {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl FromStr for QuotedString {
    type Err = InfluxLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(first_quote) = s.chars().nth(0) else {
            return Err(InfluxLineError::NoQuoteDelimiter);
        };
        let Some(last_quote) = s.chars().last() else {
            return Err(InfluxLineError::NoQuoteDelimiter);
        };
        if s.len() < 2 || first_quote != '"' || last_quote != '"' {
            return Err(InfluxLineError::NoQuoteDelimiter);
        };

        let mut parser = LinearParser::new(
            &Self::SPECIAL_CHARACTERS,
            &Self::ESCAPE_CHARACTER,
            StrayEscapes::Forbid,
        );

        s.chars()
            .skip(1)
            .take(s.len() - 2)
            .try_for_each(|character| parser.process_char(character))?;

        let name = Self::from(parser.extract()?);
        Ok(name)
    }
}

impl Display for QuotedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatter = LinearFormatter::new(&Self::SPECIAL_CHARACTERS, &Self::ESCAPE_CHARACTER);
        write!(f, "\"{}\"", formatter.chars(self).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::QuotedString;

    #[rstest::rstest]
    #[case::empty_string("\"\"", "")]
    #[case::quotes("\"\\\"string\\\" within a string\"", "\"string\" within a string")]
    #[case::backslash("\"slash \\\\ escaped\"", "slash \\ escaped")]
    fn successful_parsing(#[case] escaped_input: &str, #[case] expected_value: &str) {
        let expected_string = QuotedString::new(expected_value);

        let actual_string = QuotedString::from_str(escaped_input).expect("Must parse here");

        assert_eq!(expected_string, actual_string);
    }

    #[rstest::rstest]
    #[case::no_tokens("")]
    #[case::only_one_quote("\"")]
    #[case::one_random_symbol("a")]
    #[case::unclosed_string_on_right("\"R")]
    #[case::unclosed_string_on_left("L\"")]
    #[case::unclosed_long_string_on_right("\"I HATE PANCAKES")]
    #[case::unclosed_long_string_on_left("I LOVE PANCAKES\"")]
    #[case::quote_not_escaped("\"left \" right\"")]
    #[case::backslash_not_escaped("\"Who put \\ here?\"")]
    #[case::escaped_right_quote("\"dead\\\"")]
    fn parse_error(#[case] input: &str) {
        let _parse_error = QuotedString::from_str(input).expect_err("Must return parse error");
    }

    #[rstest::rstest]
    #[case::empty("", "\"\"")]
    #[case::single_character("a", "\"a\"")]
    #[case::single_quote("\"", "\"\\\"\"")]
    #[case::single_slash("\\", "\"\\\\\"")]
    #[case::everything("welcome, \"friend\" :\\", "\"welcome, \\\"friend\\\" :\\\\\"")]
    fn display(#[case] input: &str, #[case] expected_string: &str) {
        let actual_string = QuotedString::new(input).to_string();

        assert_eq!(expected_string, actual_string);
    }
}

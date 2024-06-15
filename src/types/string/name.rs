use std::str::FromStr;

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

#[derive(Debug, thiserror::Error)]
pub enum NameParseError {
    #[error("Failed to parse name: {0}")]
    Failed(String),
    #[error("Failed to process character at position `{1}`: {0}")]
    BadCharacter(String, usize),
    #[error(transparent)]
    Malformed(#[from] MalformedNameError),
}

impl InfluxName {
    const SPECIAL_CHARACTERS: [char; 3] = [' ', '=', ','];

    #[cfg(test)]
    fn unchecked<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self(name.into())
    }
}

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

impl PartialEq<&str> for InfluxName {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for InfluxName {
    fn eq(&self, other: &String) -> bool {
        self.0 == other.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ParserState {
    #[default]
    SeenCharacter,
    SeenEscapeCharacter,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharacterState {
    Normal,
    Special,
    Escape,
}

impl From<char> for CharacterState {
    fn from(value: char) -> Self {
        if value == '\\' {
            Self::Escape
        } else if InfluxName::SPECIAL_CHARACTERS.contains(&value) {
            Self::Special
        } else {
            Self::Normal
        }
    }
}

struct Parser {
    buffer: Vec<char>,
    state: ParserState,
}

impl Parser {
    fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(1024),
            state: ParserState::default(),
        }
    }

    fn process_char(&mut self, character: char) -> bool {
        let char_state = CharacterState::from(character);

        match (self.state, char_state) {
            (ParserState::SeenCharacter, CharacterState::Normal) => {
                self.buffer.push(character);
                self.state = ParserState::SeenCharacter
            }
            (ParserState::SeenCharacter, CharacterState::Special) => {
                self.state = ParserState::Error;
            }
            (ParserState::SeenCharacter, CharacterState::Escape) => {
                self.state = ParserState::SeenEscapeCharacter;
            }
            (ParserState::SeenEscapeCharacter, CharacterState::Normal) => {
                self.buffer.push('\\');
                self.buffer.push(character);
                self.state = ParserState::SeenCharacter;
            }
            (ParserState::SeenEscapeCharacter, CharacterState::Special) => {
                self.buffer.push(character);
                self.state = ParserState::SeenCharacter;
            }
            (ParserState::SeenEscapeCharacter, CharacterState::Escape) => {
                self.buffer.push('\\');
                self.state = ParserState::SeenCharacter;
            }
            (ParserState::Error, _) => self.state = ParserState::Error,
        }

        self.state != ParserState::Error
    }

    fn extract(mut self) -> Option<String> {
        match self.state {
            ParserState::SeenCharacter => (),
            ParserState::SeenEscapeCharacter => {
                self.buffer.push('\\');
            }
            ParserState::Error => return None,
        }

        Some(self.buffer.into_iter().collect())
    }
}

impl FromStr for InfluxName {
    type Err = NameParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser = Parser::new();

        for (index, character) in s.chars().enumerate() {
            let is_processed = parser.process_char(character);
            if !is_processed {
                return Err(NameParseError::BadCharacter(s.into(), index));
            }
        }

        let parsed = parser.extract().ok_or(NameParseError::Failed(s.into()))?;
        let name = InfluxName::try_from(parsed)?;
        Ok(name)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::InfluxName;

    #[rstest::rstest]
    #[case::no_special_characters(r#"amogus"#, InfluxName::unchecked("amogus"))]
    #[case::escaped_space(r#"hello\ man"#, InfluxName::unchecked("hello man"))]
    #[case::escaped_comma(r#"milk\,bread\,butter"#, InfluxName::unchecked("milk,bread,butter"))]
    #[case::escaped_equals(r#"a\=b"#, InfluxName::unchecked("a=b"))]
    #[case::unescaped_quote(r#"stupid"quote"#, InfluxName::unchecked("stupid\"quote"))]
    #[case::slashes_1_1(r#"a\a"#, InfluxName::unchecked(r#"a\a"#))]
    #[case::slashes_2_1(r#"a\\a"#, InfluxName::unchecked(r#"a\a"#))]
    #[case::slashes_3_2(r#"a\\\a"#, InfluxName::unchecked(r#"a\\a"#))]
    #[case::slashes_4_2(r#"a\\\\a"#, InfluxName::unchecked(r#"a\\a"#))]
    #[case::slashes_5_3(r#"a\\\\\a"#, InfluxName::unchecked(r#"a\\\a"#))]
    #[case::slashes_6_3(r#"a\\\\\\a"#, InfluxName::unchecked(r#"a\\\a"#))]
    #[case::only_slashes_1_1(r#"\"#, InfluxName::unchecked(r#"\"#))]
    #[case::only_slashes_2_1(r#"\\"#, InfluxName::unchecked(r#"\"#))]
    #[case::only_slashes_3_2(r#"\\\"#, InfluxName::unchecked(r#"\\"#))]
    #[case::only_slashes_4_2(r#"\\\\"#, InfluxName::unchecked(r#"\\"#))]
    #[case::only_slashes_5_3(r#"\\\\\"#, InfluxName::unchecked(r#"\\\"#))]
    #[case::only_slashes_6_3(r#"\\\\\\"#, InfluxName::unchecked(r#"\\\"#))]
    #[case::everything(
        r#"day\ when\ f(x\,\ y)\ \=\ 10"#,
        InfluxName::unchecked("day when f(x, y) = 10")
    )]
    #[case::unicode(r#"💀\ dead\ man\ 💀"#, InfluxName::unchecked("💀 dead man 💀"))]
    fn successful_parsing(#[case] escaped_input: &str, #[case] expected_name: InfluxName) {
        let actual_name = InfluxName::from_str(&escaped_input).expect("Must parse here");

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
        let _parse_error = InfluxName::from_str(escaped_input).expect_err("Must return error");
    }
}

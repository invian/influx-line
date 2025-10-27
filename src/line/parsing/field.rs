use crate::InfluxLineError;

use super::{exclusive_split_at, key::KeyParser, Escaped, RawKeyValuePair};

#[derive(Debug)]
pub struct FieldParser;

#[derive(Debug)]
struct FieldValueParser;

#[derive(Debug)]
struct SimpleValueParser;

#[derive(Debug)]
struct StringValueParser {
    state: ParserState,
    escaped: Escaped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldParserTail<'a> {
    Timestamp(&'a str),
    Field(&'a str),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParserState {
    Start,
    StringLeftQuote,
    StringContent,
    StringRightQuote,
}

enum Transition {
    ToTimestamp,
    ToNextField,
}

impl FieldParser {
    pub fn process<'a>(
        &self,
        line: &'a str,
    ) -> Result<(RawKeyValuePair<'a>, FieldParserTail<'a>), InfluxLineError> {
        let (key, value_tail) = KeyParser::new().process(line)?;
        let (value, tail) = FieldValueParser.process(value_tail)?;
        let pair = RawKeyValuePair { key, value };
        Ok((pair, tail))
    }
}

impl FieldValueParser {
    pub fn process<'a>(
        &self,
        line: &'a str,
    ) -> Result<(&'a str, FieldParserTail<'a>), InfluxLineError> {
        match line.chars().nth(0) {
            Some('"') => StringValueParser::new().process(line),
            Some(_) => SimpleValueParser.process(line),
            None => Err(InfluxLineError::NoValue),
        }
    }
}

impl SimpleValueParser {
    pub fn process(self, line: &str) -> Result<(&str, FieldParserTail<'_>), InfluxLineError> {
        for (index, character) in line.char_indices() {
            match character {
                '\\' => return Err(InfluxLineError::UnexpectedEscapeSymbol),
                ' ' | ',' if index == 0 => return Err(InfluxLineError::NoValue),
                ' ' => {
                    let (value, tail) = exclusive_split_at(line, index);
                    return Ok((value, FieldParserTail::Timestamp(tail)));
                }
                ',' => {
                    let (value, tail) = exclusive_split_at(line, index);
                    return Ok((value, FieldParserTail::Field(tail)));
                }
                '\n' => {
                    let (value, _) = exclusive_split_at(line, index);
                    return Ok((value, FieldParserTail::None));
                }
                _ => (),
            }
        }

        Ok((line, FieldParserTail::None))
    }
}

impl StringValueParser {
    pub fn new() -> Self {
        Self {
            state: ParserState::Start,
            escaped: Escaped::No,
        }
    }

    pub fn process(mut self, line: &str) -> Result<(&str, FieldParserTail<'_>), InfluxLineError> {
        for (index, character) in line.char_indices() {
            match self.consume_char(character)? {
                Some(Transition::ToNextField) => {
                    let (string, tail) = exclusive_split_at(line, index);
                    return Ok((string, FieldParserTail::Field(tail)));
                }
                Some(Transition::ToTimestamp) => {
                    let (string, tail) = exclusive_split_at(line, index);
                    return Ok((string, FieldParserTail::Timestamp(tail)));
                }
                None => (),
            }
        }

        match self.state {
            ParserState::StringRightQuote => Ok((line, FieldParserTail::None)),
            ParserState::Start => Err(InfluxLineError::Failed),
            ParserState::StringLeftQuote => Err(InfluxLineError::NoQuoteDelimiter),
            ParserState::StringContent => Err(InfluxLineError::NoQuoteDelimiter),
        }
    }

    pub fn consume_char(&mut self, character: char) -> Result<Option<Transition>, InfluxLineError> {
        match (self.state, self.escaped, character) {
            (ParserState::Start, _, '"') => {
                self.state = ParserState::StringLeftQuote;
                Ok(None)
            }
            (ParserState::Start, _, _) => Err(InfluxLineError::NoQuoteDelimiter),
            (ParserState::StringLeftQuote, _, '"') => {
                self.state = ParserState::StringRightQuote;
                Ok(None)
            }
            (ParserState::StringLeftQuote, _, c) => {
                if c == '\\' {
                    self.escaped = Escaped::Yes;
                }
                self.state = ParserState::StringContent;
                Ok(None)
            }
            (ParserState::StringContent, Escaped::No, '\\') => {
                self.escaped = Escaped::Yes;
                Ok(None)
            }
            (ParserState::StringContent, Escaped::No, '"') => {
                self.state = ParserState::StringRightQuote;
                Ok(None)
            }
            (ParserState::StringContent, Escaped::No, _) => Ok(None),
            (ParserState::StringContent, Escaped::Yes, '\\' | '"') => {
                self.escaped = Escaped::No;
                Ok(None)
            }
            (ParserState::StringContent, Escaped::Yes, _) => {
                Err(InfluxLineError::UnexpectedEscapeSymbol)
            }
            (ParserState::StringRightQuote, _, ',') => Ok(Some(Transition::ToNextField)),
            (ParserState::StringRightQuote, _, ' ') => Ok(Some(Transition::ToTimestamp)),
            (ParserState::StringRightQuote, _, _) => Err(InfluxLineError::SymbolsAfterClosedString),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FieldParser, FieldParserTail};

    #[rstest::rstest]
    #[case::simple_value("key=value", "key", "value", FieldParserTail::None)]
    #[case::string(
        "string=\"some string\"",
        "string",
        "\"some string\"",
        FieldParserTail::None
    )]
    #[case::empty_string("s=\"\"", "s", "\"\"", FieldParserTail::None)]
    #[case::crazy_escapes(
        "escape\\ sym\\=bol=\"\\\\prison \\\"escape\\\"\"",
        "escape\\ sym\\=bol",
        "\"\\\\prison \\\"escape\\\"\"",
        FieldParserTail::None
    )]
    #[case::timestamp_tail(
        "first=true 12345",
        "first",
        "true",
        FieldParserTail::Timestamp("12345")
    )]
    #[case::field_tail(
        "first=true,second=2u 12345",
        "first",
        "true",
        FieldParserTail::Field("second=2u 12345")
    )]
    #[case::expects_field_tail_but_will_fail_later(
        "comma=in_the_end,",
        "comma",
        "in_the_end",
        FieldParserTail::Field("")
    )]
    #[case::expects_timestamp_but_will_fail_later(
        "space=in_the_end ",
        "space",
        "in_the_end",
        FieldParserTail::Timestamp("")
    )]
    #[case::unicode(
        "he\\ just\\ 💀=fr💀,my=man",
        "he\\ just\\ 💀",
        "fr💀",
        FieldParserTail::Field("my=man")
    )]
    fn successful_field_parsing(
        #[case] input: &str,
        #[case] expected_key: &str,
        #[case] expected_value: &str,
        #[case] expected_tail: FieldParserTail,
    ) {
        let (actual_pair, actual_tail) = FieldParser.process(input).expect("Must parse here");

        assert_eq!(expected_key, actual_pair.key);
        assert_eq!(expected_value, actual_pair.value);
        assert_eq!(expected_tail, actual_tail);
    }

    #[rstest::rstest]
    #[case("123")]
    #[case("a=")]
    #[case("a=,c=d")]
    #[case(",")]
    #[case("")]
    #[case("a=\"string not closed")]
    fn field_parsing_error(#[case] input: &str) {
        let _parse_error = FieldParser.process(input).expect_err("Must fail here");
    }
}

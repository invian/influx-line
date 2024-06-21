use crate::line::InfluxLineError;

use super::{exclusive_split_at, key::KeyParser, Escaped, RawKeyValuePair};

#[derive(Debug)]
pub struct TagParser;

#[derive(Debug)]
struct TagValueParser {
    escaped: Escaped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagParserTail<'a> {
    Tag(&'a str),
    Fields(&'a str),
}

impl TagParser {
    pub fn process<'a>(
        self,
        line: &'a str,
    ) -> Result<(RawKeyValuePair<'a>, TagParserTail<'a>), InfluxLineError> {
        let (key, value_tail) = KeyParser::new().process(line)?;
        let (value, tail) = TagValueParser::new().process(value_tail)?;
        let pair = RawKeyValuePair { key, value };
        Ok((pair, tail))
    }
}

impl TagValueParser {
    pub fn new() -> Self {
        Self {
            escaped: Escaped::No,
        }
    }

    pub fn process<'a>(
        mut self,
        line: &'a str,
    ) -> Result<(&'a str, TagParserTail<'a>), InfluxLineError> {
        for (index, character) in line.char_indices() {
            match (self.escaped, character) {
                (Escaped::No, ',' | ' ') if index == 0 => {
                    return Err(InfluxLineError::NoValue)
                }
                (Escaped::No, ',') => {
                    let (value, tail) = exclusive_split_at(line, index);
                    return Ok((value, TagParserTail::Tag(tail)));
                }
                (Escaped::No, ' ') => {
                    let (value, tail) = exclusive_split_at(line, index);
                    return Ok((value, TagParserTail::Fields(tail)));
                }
                (Escaped::No, '\\') => {
                    self.escaped = Escaped::Yes;
                }
                (Escaped::No, _) => (),
                (Escaped::Yes, '\\') => {
                    self.escaped = Escaped::Yes;
                }
                (Escaped::Yes, _) => {
                    self.escaped = Escaped::No;
                }
            }
        }

        Err(InfluxLineError::NoFields)
    }
}

#[cfg(test)]
mod tests {
    use super::{TagParser, TagParserTail};

    #[rstest::rstest]
    #[case::fields_tail(
        "tag=value field=true",
        "tag",
        "value",
        TagParserTail::Fields("field=true")
    )]
    #[case::tags_tail(
        "tag1=1,tag2=2 field=true",
        "tag1",
        "1",
        TagParserTail::Tag("tag2=2 field=true")
    )]
    #[case::expects_tags_but_will_fail_later(
        "tag1=no_next_tag,",
        "tag1",
        "no_next_tag",
        TagParserTail::Tag("")
    )]
    #[case::crazy_escapes(
        "t\\ a\\=g\\,1=super\\ co\\=\\,ol,tag2=2 field=true",
        "t\\ a\\=g\\,1",
        "super\\ co\\=\\,ol",
        TagParserTail::Tag("tag2=2 field=true")
    )]
    #[case::unicode(
        "he\\ just\\ 💀=fr💀,my=man",
        "he\\ just\\ 💀",
        "fr💀",
        TagParserTail::Tag("my=man")
    )]
    fn successful_tag_parsing(
        #[case] input: &str,
        #[case] expected_key: &str,
        #[case] expected_value: &str,
        #[case] expected_tail: TagParserTail,
    ) {
        let (actual_pair, actual_tail) = TagParser.process(input).expect("Must parse here");

        assert_eq!(expected_key, actual_pair.key);
        assert_eq!(expected_value, actual_pair.value);
        assert_eq!(expected_tail, actual_tail);
    }

    #[rstest::rstest]
    #[case::empty("")]
    #[case::no_value("novalue")]
    #[case::no_value_with_equals("novalue=")]
    #[case::no_tail("tav=value")]
    fn tag_parsing_error(#[case] input: &str) {
        let _parse_error = TagParser.process(input).expect_err("Must fail here");
    }
}

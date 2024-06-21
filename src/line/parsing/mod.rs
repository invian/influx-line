mod field;
mod key;
mod measurement;
mod tag;

use field::FieldParser;
use measurement::{MeasurementParser, MeasurementTail};
use tag::{TagParser, TagParserTail};

use super::InfluxLineError;

/// Since the core lib's `split_at` is inclusive,
/// i.e., it keeps the delimiter at `index` in the second slice,
/// this wrapper returns slices with the delimiter excluded.
///
/// Only usable for ASCII chars sadly,
/// but that's the only usecase for this function anyway.
fn exclusive_split_at(s: &str, index: usize) -> (&str, &str) {
    let (left, right) = s.split_at(index);
    (left, &right[1..])
}

#[derive(Debug)]
pub struct LinearLineParser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawLine<'a> {
    measurement: &'a str,
    tags: Vec<RawKeyValuePair<'a>>,
    fields: Vec<RawKeyValuePair<'a>>,
    timestamp: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawKeyValuePair<'a> {
    key: &'a str,
    value: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Escaped {
    Yes,
    No,
}

impl LinearLineParser {
    pub fn process<'a>(self, line: &'a str) -> Result<RawLine<'a>, InfluxLineError> {
        let (measurement, measurement_tail) = MeasurementParser::new().process(line)?;

        let (tags, fields_tail) = match measurement_tail {
            MeasurementTail::Tags(tags) => self.parse_tags(tags)?,
            MeasurementTail::Fields(fields_tail) => (Vec::new(), fields_tail),
        };

        let (fields, timestamp) = self.parse_fields(fields_tail)?;

        Ok(RawLine {
            measurement,
            tags,
            fields,
            timestamp,
        })
    }

    fn parse_tags<'a>(
        &self,
        line: &'a str,
    ) -> Result<(Vec<RawKeyValuePair<'a>>, &'a str), InfluxLineError> {
        let mut pairs = Vec::new();
        let mut tail = line;

        let fields = loop {
            let (pair, new_tail) = TagParser.process(tail)?;
            pairs.push(pair);
            match new_tail {
                TagParserTail::Tag(remaining_tags) => {
                    tail = remaining_tags;
                }
                TagParserTail::Fields(fields) => break fields,
            }
        };

        Ok((pairs, fields))
    }

    fn parse_fields<'a>(
        &self,
        line: &'a str,
    ) -> Result<(Vec<RawKeyValuePair<'a>>, Option<&'a str>), InfluxLineError> {
        let mut pairs = Vec::new();
        let mut tail = line;

        let timestamp_opt = loop {
            let (pair, new_tail) = FieldParser.process(tail)?;
            pairs.push(pair);
            match new_tail {
                field::FieldParserTail::Field(field_tail) => {
                    tail = field_tail;
                }
                field::FieldParserTail::Timestamp(timestamp) => break Some(timestamp),
                field::FieldParserTail::None => break None,
            }
        };

        Ok((pairs, timestamp_opt))
    }
}

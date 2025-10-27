use crate::InfluxLineError;

use super::{Escaped, exclusive_split_at};

#[derive(Debug)]
pub struct MeasurementParser {
    escaped: Escaped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeasurementTail<'a> {
    Tags(&'a str),
    Fields(&'a str),
}

impl MeasurementParser {
    pub fn new() -> Self {
        Self {
            escaped: Escaped::No,
        }
    }

    pub fn process(mut self, line: &str) -> Result<(&str, MeasurementTail<'_>), InfluxLineError> {
        for (index, character) in line.char_indices() {
            match (self.escaped, character) {
                (Escaped::No, ',' | ' ') if index == 0 => {
                    return Err(InfluxLineError::NoMeasurement);
                }
                (Escaped::No, ',') => {
                    let (measurement, tail) = exclusive_split_at(line, index);
                    return Ok((measurement, MeasurementTail::Tags(tail)));
                }
                (Escaped::No, ' ') => {
                    let (measurement, tail) = exclusive_split_at(line, index);
                    return Ok((measurement, MeasurementTail::Fields(tail)));
                }
                (Escaped::No, '\\') => {
                    self.escaped = Escaped::Yes;
                }
                (Escaped::No, _) => (),
                (Escaped::Yes, '\\') => (),
                (Escaped::Yes, _) => {
                    self.escaped = Escaped::No;
                }
            }
        }

        Err(InfluxLineError::NoWhitespaceDelimiter)
    }
}

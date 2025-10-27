use crate::InfluxLineError;

use super::{Escaped, exclusive_split_at};

#[derive(Debug)]
pub struct KeyParser {
    escaped: Escaped,
}

impl KeyParser {
    pub fn new() -> Self {
        Self {
            escaped: Escaped::No,
        }
    }

    pub fn process(mut self, line: &str) -> Result<(&str, &str), InfluxLineError> {
        for (index, character) in line.char_indices() {
            match (self.escaped, character) {
                (Escaped::No, '\\') => {
                    self.escaped = Escaped::Yes;
                }
                (Escaped::No, ' ' | ',') => {
                    return Err(InfluxLineError::UnescapedSpecialCharacter);
                }
                (Escaped::No, '=') => {
                    return Ok(exclusive_split_at(line, index));
                }
                (Escaped::No, _) => (),
                (Escaped::Yes, _) => {
                    self.escaped = Escaped::No;
                }
            }
        }

        Err(InfluxLineError::NoValue)
    }
}

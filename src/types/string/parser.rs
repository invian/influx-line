use crate::line::InfluxLineError;

#[derive(Debug, Clone)]
pub(super) struct LinearParser<'a> {
    buffer: Vec<char>,
    escaped: EscapedBefore,
    stray_escapes: StrayEscapes,
    special_characters: &'a [char],
    escape_character: &'a char,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StrayEscapes {
    Allow,
    Forbid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EscapedBefore {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharacterType {
    Normal,
    Special,
    Escape,
}

impl<'a> LinearParser<'a> {
    const DEFAULT_BUFFER_SIZE: usize = 64;

    pub fn new(
        special_characters: &'a [char],
        escape_character: &'a char,
        stray_escapes: StrayEscapes,
    ) -> Self {
        Self {
            buffer: Vec::with_capacity(Self::DEFAULT_BUFFER_SIZE),
            escaped: EscapedBefore::No,
            stray_escapes,
            special_characters,
            escape_character,
        }
    }

    pub fn process_char(&mut self, character: char) -> Result<(), InfluxLineError> {
        match (self.escaped, self.character_type(character)) {
            (EscapedBefore::Yes, CharacterType::Normal) => {
                if self.stray_escapes == StrayEscapes::Allow {
                    self.buffer.push(*self.escape_character);
                    self.buffer.push(character);
                    self.escaped = EscapedBefore::No;
                } else {
                    return Err(InfluxLineError::UnexpectedEscapeSymbol);
                }
            }
            (EscapedBefore::Yes, _) => {
                self.buffer.push(character);
                self.escaped = EscapedBefore::No;
            }
            (EscapedBefore::No, CharacterType::Normal) => {
                self.buffer.push(character);
            }
            (EscapedBefore::No, CharacterType::Special) => {
                return Err(InfluxLineError::UnescapedSpecialCharacter);
            }
            (EscapedBefore::No, CharacterType::Escape) => {
                self.escaped = EscapedBefore::Yes;
            }
        }
        Ok(())
    }

    pub fn extract(self) -> Result<String, InfluxLineError> {
        if self.escaped == EscapedBefore::Yes {
            return Err(InfluxLineError::UnexpectedEscapeSymbol);
        }

        Ok(self.buffer.into_iter().collect())
    }

    fn character_type(&self, character: char) -> CharacterType {
        if character == *self.escape_character {
            CharacterType::Escape
        } else if self.special_characters.contains(&character) {
            CharacterType::Special
        } else {
            CharacterType::Normal
        }
    }
}

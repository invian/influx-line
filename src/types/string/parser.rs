#[derive(Debug, Clone)]
pub(super) struct LinearParser {
    buffer: Vec<char>,
    state: ParserState,
    special_characters: Vec<char>,
    escape_character: char,
}

#[derive(Debug, thiserror::Error)]
#[error("Name does not abide by naming restrictions")]
pub struct NameRestrictionError;

#[derive(Debug, thiserror::Error)]
pub enum NameParseError {
    #[error("Failed to parse name")]
    Failed,
    #[error("Special character is not escaped")]
    SpecialCharacterNotEscaped,
    #[error("Unable to process name with a trailing escape character")]
    TrailingEscapeCharacter,
    #[error(transparent)]
    Malformed(#[from] NameRestrictionError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ParserState {
    #[default]
    SeenCharacter,
    SeenEscapeCharacter,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharacterType {
    Normal,
    Special,
    Escape,
}

impl LinearParser {
    const DEFAULT_BUFFER_SIZE: usize = 1024;

    pub fn new(special_characters: Vec<char>, escape_character: char) -> Self {
        Self {
            buffer: Vec::with_capacity(Self::DEFAULT_BUFFER_SIZE),
            state: ParserState::default(),
            special_characters,
            escape_character,
        }
    }

    pub fn process_char(&mut self, character: char) -> Result<(), NameParseError> {
        match (self.state, self.character_type(character)) {
            (ParserState::SeenCharacter, CharacterType::Normal) => {
                self.buffer.push(character);
                self.state = ParserState::SeenCharacter
            }
            (ParserState::SeenCharacter, CharacterType::Special) => {
                self.state = ParserState::Error;
                return Err(NameParseError::SpecialCharacterNotEscaped);
            }
            (ParserState::SeenCharacter, CharacterType::Escape) => {
                self.state = ParserState::SeenEscapeCharacter;
            }
            (ParserState::SeenEscapeCharacter, CharacterType::Normal) => {
                self.buffer.push(self.escape_character);
                self.buffer.push(character);
                self.state = ParserState::SeenCharacter;
            }
            (ParserState::SeenEscapeCharacter, CharacterType::Special) => {
                self.buffer.push(character);
                self.state = ParserState::SeenCharacter;
            }
            (ParserState::SeenEscapeCharacter, CharacterType::Escape) => {
                self.buffer.push(self.escape_character);
                self.state = ParserState::SeenCharacter;
            }
            (ParserState::Error, _) => {
                self.state = ParserState::Error;
                return Err(NameParseError::Failed);
            }
        }

        Ok(())
    }

    pub fn extract(self) -> Result<String, NameParseError> {
        match self.state {
            ParserState::SeenCharacter => (),
            ParserState::SeenEscapeCharacter => {
                return Err(NameParseError::TrailingEscapeCharacter)
            }
            ParserState::Error => return Err(NameParseError::Failed),
        }

        Ok(self.buffer.into_iter().collect())
    }

    fn character_type(&self, character: char) -> CharacterType {
        if character == self.escape_character {
            CharacterType::Escape
        } else if self.special_characters.contains(&character) {
            CharacterType::Special
        } else {
            CharacterType::Normal
        }
    }
}

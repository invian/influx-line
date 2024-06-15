#[derive(Debug, Clone)]
pub struct LinearParser {
    buffer: Vec<char>,
    state: ParserState,
    special_characters: Vec<char>,
    escape_character: char,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParserState {
    #[default]
    SeenCharacter,
    SeenEscapeCharacter,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterType {
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

    pub fn character_type(&self, character: char) -> CharacterType {
        if character == self.escape_character {
            CharacterType::Escape
        } else if self.special_characters.contains(&character) {
            CharacterType::Special
        } else {
            CharacterType::Normal
        }
    }

    pub fn process_char(&mut self, character: char) -> bool {
        match (self.state, self.character_type(character)) {
            (ParserState::SeenCharacter, CharacterType::Normal) => {
                self.buffer.push(character);
                self.state = ParserState::SeenCharacter
            }
            (ParserState::SeenCharacter, CharacterType::Special) => {
                self.state = ParserState::Error;
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
            (ParserState::Error, _) => self.state = ParserState::Error,
        }

        self.state != ParserState::Error
    }

    pub fn extract(self) -> Option<String> {
        match self.state {
            ParserState::SeenCharacter => (),
            ParserState::SeenEscapeCharacter => return None,
            ParserState::Error => return None,
        }

        Some(self.buffer.into_iter().collect())
    }
}

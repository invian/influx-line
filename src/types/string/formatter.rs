/// Processes a string char-by-char and escapes all special symbols.
#[derive(Debug)]
pub(super) struct LinearFormatter<'a> {
    special_characters: &'a [char],
    escape_character: &'a char,
}

#[derive(Debug, Clone)]
enum FormattedCharacter {
    Escaped(char, char),
    Unescaped(char),
    Exhausted,
}

impl Iterator for FormattedCharacter {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            FormattedCharacter::Escaped(escape_character, special_symbol) => {
                let out = *escape_character;
                *self = FormattedCharacter::Unescaped(*special_symbol);
                Some(out)
            }
            FormattedCharacter::Unescaped(special_symbol) => {
                let out = *special_symbol;
                *self = FormattedCharacter::Exhausted;
                Some(out)
            }
            FormattedCharacter::Exhausted => None,
        }
    }
}

impl<'a> LinearFormatter<'a> {
    pub fn new(special_characters: &'a [char], escape_character: &'a char) -> Self {
        Self {
            special_characters,
            escape_character,
        }
    }

    pub fn chars<S>(&self, original: &'a S) -> impl Iterator<Item = char> + 'a
    where
        S: AsRef<str>,
    {
        original.as_ref().chars().flat_map(|character| {
            if self.special_characters.contains(&character) {
                FormattedCharacter::Escaped(*self.escape_character, character)
            } else {
                FormattedCharacter::Unescaped(character)
            }
        })
    }
}

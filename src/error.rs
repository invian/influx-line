/// A library level error that occurs when any failure occurs,
/// such as parse error, or invalid input in constructors or conversion traits.
#[derive(Debug, thiserror::Error)]
pub enum InfluxLineError {
    #[error("Failed to process input")]
    Failed,
    #[error("No value found")]
    NoValue,
    #[error("No measurement found")]
    NoMeasurement,
    #[error("No fields found")]
    NoFields,
    #[error("Unexpected escape symbol")]
    UnexpectedEscapeSymbol,
    #[error("Unescaped special character found")]
    UnescapedSpecialCharacter,
    #[error("Space delimiter not found")]
    NoWhitespaceDelimiter,
    #[error("Closing double quote delimiter not found")]
    NoQuoteDelimiter,
    #[error("Unexpected symbols after a closing double quote delimiter")]
    SymbolsAfterClosedString,
    #[error("Naming restriction was not met")]
    NameRestriction,
    #[error("Failed to parse Integer value")]
    IntegerNotParsed,
    #[error("Failed to parse UInteger value")]
    UIntegerNotParsed,
    #[error("Failed to parse Boolean value")]
    BooleanNotParsed,
    #[error("Failed to parse timestamp")]
    TimestampNotParsed,
    #[error("Failed to parse field value as any of the expected types")]
    BadValue,
    #[error("Timestamp not constructed: DateTime out of range")]
    DateTimeOutOfRange,
}

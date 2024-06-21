mod hash_like;
mod parsing;

use std::str::FromStr;

use chrono::{DateTime, Utc};
use hash_like::KeyValueStorage;
use parsing::LinearLineParser;

use crate::{InfluxValue, KeyName, MeasurementName};

/// Implements InfluxDB Line Protocol V2.
///
/// Described [here](https://docs.influxdata.com/influxdb/v2/reference/syntax/line-protocol/).
#[derive(Debug, Clone, PartialEq)]
pub struct InfluxLine {
    /// Required measurement name.
    measurement: MeasurementName,
    /// The original name `Tag Set` is not adapted for simplicity.
    tags: KeyValueStorage<KeyName>,
    /// The original name `Field Set` is not adapted for simplicity.
    fields: KeyValueStorage<InfluxValue>,
    /// [`DateTime`] sounds more readable for a timestamp.
    timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, thiserror::Error)]
pub enum InfluxLineError {
    #[error("Failed to parse special character")]
    Failed,
    #[error("No value found")]
    NoValue,
    #[error("Failed to find measurement")]
    NoMeasurement,
    #[error("Failed to find fields set")]
    NoFields,
    #[error("Unexpected escape symbol")]
    UnexpectedEscapeSymbol,
    #[error("Unescaped special character found")]
    UnescapedSpecialCharacter,
    #[error("Space delimiter not found")]
    NoWhitespaceDelimiter,
    #[error("Equals sign delimiter not found")]
    NoEqualsDelimiter,
    #[error("Comma delimiter not found")]
    NoCommaDelimiter,
    #[error("Closing double quote delimiter not found")]
    NoQuoteDelimiter,
    #[error("Naming restriction was not met")]
    NameRestriction,
    #[error("Failed to parse key")]
    KeyNotParsed,
    #[error("Failed to parse Integer value")]
    IntegerNotParsed,
    #[error("Failed to parse UInteger value")]
    UIntegerNotParsed,
    #[error("Failed to parse Boolean value")]
    BooleanNotParsed,
    #[error("Failed to parse timestamp")]
    TimestampNotParsed,
}

impl InfluxLine {
    pub fn full<DT>(
        measurement: MeasurementName,
        tags: impl IntoIterator<Item = (KeyName, KeyName)>,
        fields: impl IntoIterator<Item = (KeyName, InfluxValue)>,
        timestamp: Option<DT>,
    ) -> Self
    where
        DT: Into<DateTime<Utc>>,
    {
        Self {
            measurement,
            tags: tags.into_iter().collect(),
            fields: fields.into_iter().collect(),
            timestamp: timestamp.map(|ts| ts.into()),
        }
    }

    pub fn new<V>(measurement: MeasurementName, field: KeyName, value: V) -> Self
    where
        V: Into<InfluxValue>,
    {
        let fields = [(field, value.into())].into_iter().collect();
        Self {
            measurement,
            tags: KeyValueStorage::new(),
            fields,
            timestamp: None,
        }
    }

    pub fn try_new<M, K, V>(measurement: M, field: K, value: V) -> Result<Self, InfluxLineError>
    where
        M: TryInto<MeasurementName, Error = InfluxLineError>,
        K: TryInto<KeyName, Error = InfluxLineError>,
        V: Into<InfluxValue>,
    {
        let fields = [(field.try_into()?, value.into())].into_iter().collect();
        Ok(Self {
            measurement: measurement.try_into()?,
            tags: KeyValueStorage::new(),
            fields,
            timestamp: None,
        })
    }

    pub fn measurement(&self) -> &MeasurementName {
        &self.measurement
    }

    pub fn tag<S>(&self, name: S) -> Option<&KeyName>
    where
        S: AsRef<str>,
    {
        self.tags.get(name)
    }

    pub fn field<S>(&self, name: S) -> Option<&InfluxValue>
    where
        S: AsRef<str>,
    {
        self.fields.get(name)
    }

    pub fn timestamp(&self) -> Option<DateTime<Utc>> {
        self.timestamp
    }

    /// # Examples
    ///
    /// ```rust
    /// use chrono::Utc;
    /// use influx_line::*;
    ///
    /// let some_time = Utc::now();
    /// let measurement = MeasurementName::new("human").unwrap();
    /// let field = KeyName::new("age").unwrap();
    /// let line = InfluxLine::new(measurement, field, 15)
    ///     .with_timestamp(some_time);
    ///
    /// assert_eq!(some_time, line.timestamp().unwrap());
    /// ```
    pub fn with_timestamp<T>(mut self, timestamp: T) -> Self
    where
        T: Into<DateTime<Utc>>,
    {
        self.timestamp.replace(timestamp.into());
        self
    }

    /// # Examples
    ///
    /// ## No tags yet
    ///
    /// ```rust
    /// use influx_line::*;
    ///
    /// let measurement = MeasurementName::new("human").unwrap();
    /// let field = KeyName::new("age").unwrap();
    /// let line = InfluxLine::new(measurement, field, 15);
    ///
    /// assert_eq!(line.tag("there are no tags yet, buddy"), None);
    /// ```
    ///
    /// ## Adding and overriding tags
    ///
    /// ```rust
    /// use influx_line::*;
    ///
    /// let measurement = MeasurementName::new("human").unwrap();
    /// let field = KeyName::new("age").unwrap();
    /// let line = InfluxLine::new(measurement, field, 15)
    ///     .with_tag(KeyName::new("club").unwrap(), KeyName::new("art").unwrap())
    ///     .with_tag(KeyName::new("location").unwrap(), KeyName::new("siberia").unwrap())
    ///     .with_tag(KeyName::new("club").unwrap(), KeyName::new("sports").unwrap());
    ///
    /// assert_eq!(line.tag("location").unwrap().as_ref(), "siberia");
    /// assert_eq!(line.tag("club").unwrap().as_ref(), "sports");
    /// assert_eq!(line.tag("not added yet lol"), None);
    /// ```
    pub fn with_tag(mut self, tag: KeyName, value: KeyName) -> Self {
        self.tags.put(tag, value);
        self
    }

    pub fn try_with_tag<K, V>(mut self, tag: K, value: V) -> Result<Self, InfluxLineError>
    where
        K: TryInto<KeyName, Error = InfluxLineError>,
        V: TryInto<KeyName, Error = InfluxLineError>,
    {
        self.tags.put(tag.try_into()?, value.try_into()?);
        Ok(self)
    }

    /// # Examples
    ///
    /// ## At least one field is mandatory
    ///
    /// ```rust
    /// use influx_line::*;
    ///
    /// let measurement = MeasurementName::new("human").unwrap();
    /// let field = KeyName::new("age").unwrap();
    /// let line = InfluxLine::new(measurement, field, 15);
    ///
    /// assert_eq!(line.field("height"), None);
    /// assert_eq!(line.field("age").cloned().unwrap(), 15.into());
    /// ```
    ///
    /// ## Adding and overriding fields
    ///
    /// ```rust
    /// use influx_line::*;
    ///
    /// let measurement = MeasurementName::new("human").unwrap();
    /// let field = KeyName::new("age").unwrap();
    /// let line = InfluxLine::new(measurement, field, 15)
    ///     .with_field(KeyName::new("height").unwrap(), 1.82)
    ///     .with_field(KeyName::new("age").unwrap(), 55)
    ///     .with_field(KeyName::new("is_epic").unwrap(), true)
    ///     .with_field(KeyName::new("name").unwrap(), "armstrong");
    ///
    /// assert_eq!(line.field("height").cloned().unwrap(), 1.82.into() );
    /// assert_eq!(line.field("age").cloned().unwrap(), 55.into());
    /// assert_eq!(line.field("is_epic").cloned().unwrap(), true.into());
    /// assert_eq!(line.field("name").cloned().unwrap(), "armstrong".into());
    /// assert_eq!(line.field("non-existent"), None);
    /// ```
    pub fn with_field<V>(mut self, field: KeyName, value: V) -> Self
    where
        V: Into<InfluxValue>,
    {
        self.fields.put(field, value.into());
        self
    }

    pub fn try_with_field<K, V>(mut self, field: K, value: V) -> Result<Self, InfluxLineError>
    where
        K: TryInto<KeyName, Error = InfluxLineError>,
        V: Into<InfluxValue>,
    {
        self.fields.put(field.try_into()?, value.into());
        Ok(self)
    }
}

impl FromStr for InfluxLine {
    type Err = InfluxLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw_line = LinearLineParser.process(s)?;
        raw_line.try_into()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{InfluxLine, Timestamp};

    #[rstest::rstest]
    #[case::minimal(
        "measurement field1=228u",
        InfluxLine::try_new("measurement", "field1", 228 as u32).unwrap()
    )]
    #[case::full(
        "human,language=ru,location=siberia age=25u,is\\ epic=true,balance=-15.57,name=\"Egorka\" 1704067200000000000",
        InfluxLine::try_new("human", "age", 25 as u32)
            .and_then(|l| l.try_with_field("is epic", true))
            .and_then(|l| l.try_with_field("balance", -15.57))
            .and_then(|l| l.try_with_field("name", "Egorka"))
            .and_then(|l| l.try_with_tag("language", "ru"))
            .and_then(|l| l.try_with_tag("location", "siberia"))
            .map(|l| l.with_timestamp(Timestamp::from(1704067200000000000 as i64)))
            .unwrap()
    )]
    fn successful_line_parsing(#[case] input: &str, #[case] expected_line: InfluxLine) {
        let actual_line = InfluxLine::from_str(input).expect("Must parse here");

        assert_eq!(expected_line, actual_line);
    }

    #[rstest::rstest]
    #[case::empty("")]
    #[case::no_fields("measurement,tag1=tag1,tag2=tag2")]
    #[case::no_fields_but_yes_timestamp("measurement,tag1=tag1,tag2=tag2 123456789")]
    #[case::no_escape("measure ment,tag1=tag1,tag2=tag2 field1=1.0 12345")]
    #[case::bad_field_value("measurement,tag1=tag1,tag2=tag2 field1=not\\ a\\ string 12345")]
    #[case::bad_timestamp("measurement field1=1.00 timestamp_here")]
    fn line_parsing_error(#[case] input: &str) {
        let _parse_error = InfluxLine::from_str(input).expect_err("Must fail here");
    }
}

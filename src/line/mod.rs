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

    pub fn try_new<M, K, V>(
        measurement: M,
        field: KeyName,
        value: V,
    ) -> Result<Self, InfluxLineError>
    where
        M: TryInto<MeasurementName, Error = InfluxLineError>,
        V: Into<InfluxValue>,
    {
        let fields = [(field, value.into())].into_iter().collect();
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

    use crate::{InfluxLine, InfluxValue, KeyName, MeasurementName, Timestamp};

    use super::hash_like::KeyValuePair;

    type TagPair = KeyValuePair<KeyName>;
    type FieldPair = KeyValuePair<InfluxValue>;

    #[rstest::fixture]
    fn timestamp() -> Timestamp {
        (1704067200000000000 as i64).into()
    }

    #[rstest::fixture]
    fn human_measurement() -> MeasurementName {
        "human"
            .try_into()
            .expect("Must be a valid measurement name")
    }

    #[rstest::fixture]
    fn ru_language_tag() -> TagPair {
        TagPair {
            key: "language".try_into().expect("Must be a valid tag key"),
            value: "ru".try_into().expect("Must be a valid tag value"),
        }
    }

    #[rstest::fixture]
    fn en_language_tag() -> TagPair {
        TagPair {
            key: "language".try_into().expect("Must be a valid tag key"),
            value: "en".try_into().expect("Must be a valid tag value"),
        }
    }

    #[rstest::fixture]
    fn location_tag() -> TagPair {
        TagPair {
            key: "location".try_into().expect("Must be a valid tag key"),
            value: "siberia".try_into().expect("Must be a valid tag value"),
        }
    }

    #[rstest::fixture]
    fn age_field() -> FieldPair {
        FieldPair {
            key: "age".try_into().expect("Must be a valid field key"),
            value: (25 as u32).into(),
        }
    }

    #[rstest::fixture]
    fn epicness_field() -> FieldPair {
        FieldPair {
            key: "is epic".try_into().expect("Must be a valid field key"),
            value: true.into(),
        }
    }

    #[rstest::fixture]
    fn balance_field() -> FieldPair {
        FieldPair {
            key: "balance".try_into().expect("Must be a valid field key"),
            value: (-15.57).into(),
        }
    }

    #[rstest::fixture]
    fn name_field() -> FieldPair {
        FieldPair {
            key: "name".try_into().expect("Must be a valid field key"),
            value: "Egorka".into(),
        }
    }

    #[rstest::rstest]
    fn successful_full_line_parsing(
        human_measurement: MeasurementName,
        ru_language_tag: TagPair,
        location_tag: TagPair,
        age_field: FieldPair,
        epicness_field: FieldPair,
        balance_field: FieldPair,
        name_field: FieldPair,
        timestamp: Timestamp,
    ) {
        let input = "human,language=ru,location=siberia age=25u,is\\ epic=true,balance=-15.57,name=\"Egorka\" 1704067200000000000";
        let expected_line = InfluxLine::new(human_measurement, age_field.key, age_field.value)
            .with_field(epicness_field.key, epicness_field.value)
            .with_field(balance_field.key, balance_field.value)
            .with_field(name_field.key, name_field.value)
            .with_tag(ru_language_tag.key, ru_language_tag.value)
            .with_tag(location_tag.key, location_tag.value)
            .with_timestamp(timestamp);

        let actual_line = InfluxLine::from_str(input).expect("Must parse here");

        assert_eq!(expected_line, actual_line);
    }
}

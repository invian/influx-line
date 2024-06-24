mod hash_like;
mod parsing;

use std::fmt::Display;
use std::str::FromStr;

use hash_like::KeyValueStorage;
use parsing::LinearLineParser;

use crate::{InfluxLineError, InfluxValue, KeyName, MeasurementName, Timestamp};

/// Implements InfluxDB Line Protocol V2
/// described [here](https://docs.influxdata.com/influxdb/v2/reference/syntax/line-protocol/).
///
/// A minimal possible Line can be constructed via [`Self::new`] / [`Self::try_new`].
/// Other builder methods can add extra tags/fields and timestamp:
///
/// - [`Self::with_tag`] / [`Self::try_with_tag`]
/// - [`Self::with_field`] / [`Self::try_with_field`]
/// - [`Self::with_timestamp`] / [`Self::try_with_timestamp`]
///
/// Additionally, [`std::str::FromStr`] and [`std::fmt::Display`] implementations
/// allow for parsing and formatting the Line
/// as per Line Protocol described in the InfluxDB docs.
/// Escaping is done automatically under the hood,
/// so that consumers can work with raw values with comfort and style.
#[derive(Debug, Clone, PartialEq)]
pub struct InfluxLine {
    measurement: MeasurementName,
    /// The original name `Tag Set` is not adapted for simplicity.
    tags: KeyValueStorage<KeyName>,
    /// The original name `Field Set` is not adapted for simplicity.
    fields: KeyValueStorage<InfluxValue>,
    timestamp: Option<Timestamp>,
}

impl InfluxLine {
    /// Creates a Line from all of its components.
    pub fn full<DT>(
        measurement: MeasurementName,
        tags: impl IntoIterator<Item = (KeyName, KeyName)>,
        fields: impl IntoIterator<Item = (KeyName, InfluxValue)>,
        timestamp: Option<DT>,
    ) -> Result<Self, InfluxLineError>
    where
        DT: Into<Timestamp>,
    {
        let actual_fields: KeyValueStorage<InfluxValue> = fields.into_iter().collect();
        if actual_fields.is_empty() {
            return Err(InfluxLineError::NoFields);
        }

        Ok(Self {
            measurement,
            tags: tags.into_iter().collect(),
            fields: actual_fields,
            timestamp: timestamp.map(|ts| ts.into()),
        })
    }

    /// Creates a minimal allowed Line that may later be filled with more data.
    /// Works with checked and verified values only.
    ///
    /// Has a fallible counterpart: [`Self::try_new`].
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

    /// Creates a minimal allowed Line that may later be filled with more data.
    /// Does type conversions by itself and reports errors if any.
    ///
    /// Behaves the same as its infallible counterpart: [`Self::new`].
    pub fn try_new<M, K, V>(measurement: M, field: K, value: V) -> Result<Self, InfluxLineError>
    where
        M: TryInto<MeasurementName, Error = InfluxLineError>,
        K: TryInto<KeyName, Error = InfluxLineError>,
        V: Into<InfluxValue>,
    {
        Ok(Self::new(measurement.try_into()?, field.try_into()?, value))
    }

    /// Returns a measurement name.
    pub fn measurement(&self) -> &MeasurementName {
        &self.measurement
    }

    /// Returns a tag value given the tag key.
    pub fn tag<S>(&self, name: S) -> Option<&KeyName>
    where
        S: AsRef<str>,
    {
        self.tags.get(name)
    }

    /// Returns an iterator over tag key-value pairs.
    pub fn tags(&self) -> impl Iterator<Item = (&KeyName, &KeyName)> {
        self.tags.iter()
    }

    /// Returns a field value given the field key.
    pub fn field<S>(&self, name: S) -> Option<&InfluxValue>
    where
        S: AsRef<str>,
    {
        self.fields.get(name)
    }

    /// Returns an iterator over field key-value pairs.
    pub fn fields(&self) -> impl Iterator<Item = (&KeyName, &InfluxValue)> {
        self.fields.iter()
    }

    /// Returns the timestamp value.
    pub fn timestamp(&self) -> Option<Timestamp> {
        self.timestamp
    }

    /// Adds a timestamp to the line, overriding the previous value.
    ///
    /// Expects a dedicated [`Timestamp`] type.
    ///
    /// Has a fallible counterpart: [`Self::try_with_tag`].
    /// That one might be more convenient when working with [`DateTime`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use chrono::{DateTime, Utc};
    /// use influx_line::*;
    ///
    /// let timestamp = Timestamp::from(1704069900000000000 as i64);
    /// let datetime: DateTime<Utc> = timestamp.into();
    ///
    /// let measurement = MeasurementName::new("human").unwrap();
    /// let field = KeyName::new("age").unwrap();
    /// let line = InfluxLine::new(measurement, field, 15)
    ///     .with_timestamp(timestamp);
    ///
    /// assert_eq!(timestamp, line.timestamp().unwrap());
    /// assert_eq!(datetime, line.timestamp().unwrap().into());
    /// ```
    pub fn with_timestamp<T>(mut self, timestamp: T) -> Self
    where
        T: Into<Timestamp>,
    {
        self.timestamp.replace(timestamp.into());
        self
    }

    /// A nice way to add a timestamp from [`DateTime`].
    ///
    /// This method is fallible because sometimes
    /// [`DateTime`] may be out of range and return `None` nanoseconds.
    ///
    /// Behaves the same as its infallible counterpart: [`Self::with_timestamp`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// use chrono::{DateTime, Utc};
    /// use influx_line::*;
    ///
    /// let time = Utc::now();
    /// let line = InfluxLine::try_new("human", "age", 15)
    ///     .and_then(|line| line.try_with_timestamp(time))
    ///     .unwrap();
    ///
    /// assert_eq!(time, line.timestamp().unwrap().into());
    /// ```
    pub fn try_with_timestamp<T>(mut self, timestamp: T) -> Result<Self, InfluxLineError>
    where
        T: TryInto<Timestamp, Error = InfluxLineError>,
    {
        self.timestamp.replace(timestamp.try_into()?);
        Ok(self)
    }

    /// Adds a tag to the Line.
    /// Overrides the existing tag, but does not place it in the end.
    ///
    /// Works with checked and verified values only.
    ///
    /// Has a fallible counterpart: [`Self::try_with_tag`].
    /// That one might be more convenient sometimes.
    ///
    /// # Examples
    ///
    /// ## No tags yet
    ///
    /// ```rust
    /// use influx_line::*;
    ///
    /// let line = InfluxLine::try_new("human", "age", 15).unwrap();
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

    /// A convenience method for adding tags from raw unchecked types.
    /// Attempts fallible conversions by itself and reports errors if any.
    ///
    /// Other than that, works the same as its infallible counterpart: [`Self::with_tag`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influx_line::*;
    ///
    /// let line = InfluxLine::try_new("human", "age", 15)
    ///     .and_then(|line| line.try_with_tag("club", "art"))
    ///     .and_then(|line| line.try_with_tag("location", "siberia"))
    ///     .and_then(|line| line.try_with_tag("club", "sports"))
    ///     .unwrap();
    ///
    /// assert_eq!(line.tag("location").unwrap().as_ref(), "siberia");
    /// assert_eq!(line.tag("club").unwrap().as_ref(), "sports");
    /// assert_eq!(line.tag("not added yet lol"), None);
    /// ```
    pub fn try_with_tag<K, V>(mut self, tag: K, value: V) -> Result<Self, InfluxLineError>
    where
        K: TryInto<KeyName, Error = InfluxLineError>,
        V: TryInto<KeyName, Error = InfluxLineError>,
    {
        self.tags.put(tag.try_into()?, value.try_into()?);
        Ok(self)
    }

    /// Adds a field to the Line.
    /// Overrides the existing field, but does not place it in the end.
    ///
    /// Works with checked and verified values only.
    ///
    /// Has a fallible counterpart: [`Self::try_with_field`].
    /// That one might be more convenient sometimes.
    ///
    /// # Examples
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
    /// assert_eq!(line.field("height").cloned().unwrap(), 1.82.into());
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

    /// A convenience method for adding fields from raw unchecked types.
    /// Attempts fallible conversions by itself and reports errors if any.
    ///
    /// Other than that, works the same as its infallible counterpart: [`Self::with_field`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influx_line::*;
    ///
    /// let line = InfluxLine::try_new("human", "age", 15)
    ///     .and_then(|line| line.try_with_field("height", 1.82))
    ///     .and_then(|line| line.try_with_field("age", 55))
    ///     .and_then(|line| line.try_with_field("is_epic", true))
    ///     .and_then(|line| line.try_with_field("name", "armstrong"))
    ///     .unwrap();
    ///
    /// assert_eq!(line.field("height").cloned().unwrap(), 1.82.into());
    /// assert_eq!(line.field("age").cloned().unwrap(), 55.into());
    /// assert_eq!(line.field("is_epic").cloned().unwrap(), true.into());
    /// assert_eq!(line.field("name").cloned().unwrap(), "armstrong".into());
    /// assert_eq!(line.field("non-existent"), None);
    /// ```
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

impl Display for InfluxLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.measurement)?;

        for (key, value) in self.tags.iter() {
            write!(f, ",{}={}", key, value)?;
        }

        for (index, (key, value)) in self.fields.iter().enumerate() {
            if index != 0 {
                write!(f, ",{}={}", key, value)?;
            } else {
                write!(f, " {}={}", key, value)?;
            }
        }

        if let Some(timestamp) = self.timestamp {
            write!(f, " {}", timestamp)?;
        }

        Ok(())
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
    #[case::minimal_with_newline(
        "measurement field1=228u\n",
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
    fn display_line(#[case] expected_str: &str, #[case] line: InfluxLine) {
        let actual_str = line.to_string();

        assert_eq!(expected_str, actual_str);
    }
}

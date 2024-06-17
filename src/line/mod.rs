use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::{InfluxValue, KeyName, MeasurementName, NameRestrictionError};

/// Implements InfluxDB Line Protocol V2.
///
/// Described [here](https://docs.influxdata.com/influxdb/v2/reference/syntax/line-protocol/).
#[derive(Debug, Clone)]
pub struct InfluxLine {
    /// Required measurement name.
    measurement: MeasurementName,
    /// The original name `Tag Set` is not adapted for simplicity.
    tags: HashMap<KeyName, KeyName>,
    /// The original name `Field Set` is not adapted for simplicity.
    fields: HashMap<KeyName, InfluxValue>,
    /// [`DateTime`] sounds more readable for a timestamp.
    timestamp: Option<DateTime<Utc>>,
}

impl InfluxLine {
    pub fn new<V>(measurement: MeasurementName, field: KeyName, value: V) -> Self
    where
        V: Into<InfluxValue>,
    {
        let fields = [(field, value.into())].into_iter().collect();
        Self {
            measurement,
            tags: HashMap::new(),
            fields,
            timestamp: None,
        }
    }

    pub fn try_new<M, K, V>(
        measurement: M,
        field: KeyName,
        value: V,
    ) -> Result<Self, NameRestrictionError>
    where
        M: TryInto<MeasurementName, Error = NameRestrictionError>,
        V: Into<InfluxValue>,
    {
        let fields = [(field, value.into())].into_iter().collect();
        Ok(Self {
            measurement: measurement.try_into()?,
            tags: HashMap::new(),
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
        self.tags
            .iter()
            .find_map(|(key, value)| (key.as_str() == name.as_ref()).then_some(value))
    }

    pub fn field<S>(&self, name: S) -> Option<&InfluxValue>
    where
        S: AsRef<str>,
    {
        self.fields
            .iter()
            .find_map(|(key, value)| (key.as_str() == name.as_ref()).then_some(value))
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
        self.tags.insert(tag.into(), value.into());
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
        self.fields.insert(field, value.into());
        self
    }
}

use std::{collections::HashMap, marker::PhantomData};

use chrono::{DateTime, Utc};

use crate::types::InfluxValue;

/// Implements InfluxDB Line Protocol V2.
///
/// Described [here](https://docs.influxdata.com/influxdb/v2/reference/syntax/line-protocol/).
#[derive(Debug, Clone)]
pub struct InfluxLine {
    pub measurement: String,
    /// The original name `Tag Set` is not adapted for simplicity.
    pub tags: HashMap<String, String>,
    /// The original name `Field Set` is not adapted for simplicity.
    pub fields: HashMap<String, InfluxValue>,
    /// [`DateTime`] sounds more readable for a timestamp
    pub timestamp: Option<DateTime<Utc>>,
    _phantom: PhantomData<()>,
}

impl InfluxLine {
    pub fn new<S1, S2, V>(measurement: S1, field: S2, value: V) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        V: Into<InfluxValue>,
    {
        let fields = [(field.into(), value.into())].into_iter().collect();
        Self {
            measurement: measurement.into(),
            tags: HashMap::new(),
            fields,
            timestamp: None,
            _phantom: PhantomData,
        }
    }

    /// # Examples
    ///
    /// ```rust
    /// use chrono::Utc;
    /// use influx_line::*;
    ///
    /// let some_time = Utc::now();
    /// let line = InfluxLine::new("human", "age", 15)
    ///     .with_timestamp(some_time);
    ///
    /// assert_eq!(some_time, line.timestamp.unwrap());
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
    /// let line = InfluxLine::new("human", "age", 15);
    ///
    /// assert!(line.tags.is_empty());
    /// assert_eq!(None, line.tags.get("there are no tags yet, buddy"));
    /// ```
    ///
    /// ## Adding and overriding tags
    ///
    /// ```rust
    /// use influx_line::*;
    ///
    /// let line = InfluxLine::new("human", "age", 15)
    ///     .with_tag("club", "art")
    ///     .with_tag("location", "siberia")
    ///     .with_tag("club", "sports");
    ///
    /// assert_eq!("siberia", line.tags.get("location").unwrap());
    /// assert_eq!("sports", line.tags.get("club").unwrap());
    /// assert_eq!(None, line.tags.get("not added yet lol"));
    /// ```
    pub fn with_tag<S1, S2>(mut self, tag: S1, value: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
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
    /// let line = InfluxLine::new("human", "age", 15);
    ///
    /// assert_eq!(1, line.fields.len());
    /// assert_eq!(None, line.fields.get("height"));
    /// assert_eq!(InfluxValue::Integer(15), line.fields.get("age").cloned().unwrap());
    /// ```
    ///
    /// ## Adding and overriding fields
    ///
    /// ```rust
    /// use influx_line::*;
    ///
    /// let line = InfluxLine::new("human", "age", 15)
    ///     .with_field("height", 1.82)
    ///     .with_field("age", 55)
    ///     .with_field("is_epic", true)
    ///     .with_field("name", "armstrong");
    ///
    /// assert_eq!(InfluxValue::Float(1.82), line.fields.get("height").cloned().unwrap());
    /// assert_eq!(InfluxValue::Integer(55), line.fields.get("age").cloned().unwrap());
    /// assert_eq!(InfluxValue::Boolean(true), line.fields.get("is_epic").cloned().unwrap());
    /// assert_eq!(InfluxValue::String("armstrong".into()), line.fields.get("name").cloned().unwrap());
    /// assert_eq!(None, line.fields.get("non-existent"));
    /// ```
    pub fn with_field<S, V>(mut self, field: S, value: V) -> Self
    where
        S: Into<String>,
        V: Into<InfluxValue>,
    {
        self.fields.insert(field.into(), value.into());
        self
    }
}

use std::{collections::HashMap, marker::PhantomData};

use chrono::{DateTime, Utc};

use crate::{InfluxValue, MeasurementName, NameRestrictionError};

/// Implements InfluxDB Line Protocol V2.
///
/// Described [here](https://docs.influxdata.com/influxdb/v2/reference/syntax/line-protocol/).
#[derive(Debug, Clone)]
pub struct InfluxLine {
    /// Required measurement name.
    pub measurement: MeasurementName,
    /// The original name `Tag Set` is not adapted for simplicity.
    pub tags: HashMap<String, String>,
    /// The original name `Field Set` is not adapted for simplicity.
    pub fields: HashMap<String, InfluxValue>,
    /// [`DateTime`] sounds more readable for a timestamp.
    pub timestamp: Option<DateTime<Utc>>,
    /// Fancy constructors are preferable for safery.
    _phantom: PhantomData<()>,
}

impl InfluxLine {
    pub fn new<K, V>(measurement: MeasurementName, field: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<InfluxValue>,
    {
        let fields = [(field.into(), value.into())].into_iter().collect();
        Self {
            measurement,
            tags: HashMap::new(),
            fields,
            timestamp: None,
            _phantom: PhantomData,
        }
    }

    pub fn try_new<M, K, V>(
        measurement: M,
        field: K,
        value: V,
    ) -> Result<Self, NameRestrictionError>
    where
        M: TryInto<MeasurementName, Error = NameRestrictionError>,
        K: Into<String>,
        V: Into<InfluxValue>,
    {
        let fields = [(field.into(), value.into())].into_iter().collect();
        Ok(Self {
            measurement: measurement.try_into()?,
            tags: HashMap::new(),
            fields,
            timestamp: None,
            _phantom: PhantomData,
        })
    }

    /// # Examples
    ///
    /// ```rust
    /// use chrono::Utc;
    /// use influx_line::*;
    ///
    /// let some_time = Utc::now();
    /// let line = InfluxLine::new("human".try_into().unwrap(), "age", 15)
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
    /// let line = InfluxLine::new("human".try_into().unwrap(), "age", 15);
    ///
    /// assert!(line.tags.is_empty());
    /// assert_eq!(line.tags.get("there are no tags yet, buddy"), None);
    /// ```
    ///
    /// ## Adding and overriding tags
    ///
    /// ```rust
    /// use influx_line::*;
    ///
    /// let line = InfluxLine::new("human".try_into().unwrap(), "age", 15)
    ///     .with_tag("club", "art")
    ///     .with_tag("location", "siberia")
    ///     .with_tag("club", "sports");
    ///
    /// assert_eq!(line.tags.get("location").unwrap(), "siberia");
    /// assert_eq!(line.tags.get("club").unwrap(), "sports");
    /// assert_eq!(line.tags.get("not added yet lol"), None);
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
    /// let line = InfluxLine::new("human".try_into().unwrap(), "age", 15);
    ///
    /// assert_eq!(line.fields.len(), 1);
    /// assert_eq!(line.fields.get("height"), None);
    /// assert_eq!(line.fields.get("age").cloned().unwrap(), 15.into());
    /// ```
    ///
    /// ## Adding and overriding fields
    ///
    /// ```rust
    /// use influx_line::*;
    ///
    /// let line = InfluxLine::new("human".try_into().unwrap(), "age", 15)
    ///     .with_field("height", 1.82)
    ///     .with_field("age", 55)
    ///     .with_field("is_epic", true)
    ///     .with_field("name", "armstrong");
    ///
    /// assert_eq!(line.fields.get("height").cloned().unwrap(), 1.82.into() );
    /// assert_eq!(line.fields.get("age").cloned().unwrap(), 55.into());
    /// assert_eq!(line.fields.get("is_epic").cloned().unwrap(), true.into() );
    /// assert_eq!(line.fields.get("name").cloned().unwrap(), "armstrong".into());
    /// assert_eq!(line.fields.get("non-existent"), None);
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

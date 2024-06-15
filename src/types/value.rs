use crate::{InfluxInteger, InfluxUInteger};

#[derive(Debug, Clone, PartialEq, derive_more::From)]
pub enum InfluxValue {
    #[from(types(f32))]
    Float(f64),
    #[from(types(i8, i16, i32, i64))]
    Integer(InfluxInteger),
    #[from(types(u8, u16, u32, u64))]
    UInteger(InfluxUInteger),
    #[from]
    Boolean(bool),
    #[from]
    String(String),
}

impl From<&str> for InfluxValue {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

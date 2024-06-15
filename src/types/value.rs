#[derive(Debug, Clone, PartialEq, derive_more::From)]
pub enum InfluxValue {
    #[from(types(f32))]
    Float(f64),
    #[from(types(u8, i8, u16, i16, u32, i32))]
    Integer(i64),
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

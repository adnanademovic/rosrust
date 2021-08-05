use std::collections::HashMap;

pub type MessageValue = HashMap<String, Value>;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    String(String),
    Time { sec: u32, nsec: u32 },
    Duration { sec: i32, nsec: i32 },
    Array(Vec<Value>),
    Message(MessageValue),
}

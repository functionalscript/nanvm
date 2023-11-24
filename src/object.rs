use crate::{container::Info, value::Value};

pub struct ObjectHeader(usize);

impl Info for ObjectHeader {
    type Item = (Value, Value);
}

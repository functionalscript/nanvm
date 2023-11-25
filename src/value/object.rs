use crate::{container::Info, value::unknown::Value};

pub struct ObjectHeader();

impl Info for ObjectHeader {
    type Item = (Value, Value);
}

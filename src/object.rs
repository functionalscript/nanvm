use crate::{container::Info, value::Value};

pub struct ObjectHeader();

impl Info for ObjectHeader {
    type Item = (Value, Value);
}

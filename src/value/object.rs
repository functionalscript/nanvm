use crate::{
    container::{ContainerRef, Info},
    value::unknown::Value,
};

pub struct ObjectHeader();

impl Info for ObjectHeader {
    type Item = (Value, Value);
}

pub type ObjectRef = ContainerRef<ObjectHeader>;

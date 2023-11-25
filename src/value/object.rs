use crate::{
    container::{ContainerRef, Info},
    value::unknown::Unknown,
};

pub struct ObjectHeader();

impl Info for ObjectHeader {
    type Item = (Unknown, Unknown);
}

pub type ObjectRef = ContainerRef<ObjectHeader>;

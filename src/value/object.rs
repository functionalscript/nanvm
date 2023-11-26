use crate::{
    container::{Container, ContainerRef, Info},
    value::unknown::Unknown,
};

pub struct ObjectHeader();

impl Info for ObjectHeader {
    type Item = (Unknown, Unknown);
}

pub type ObjectContainer = Container<ObjectHeader>;

pub type ObjectRef = ContainerRef<ObjectHeader>;

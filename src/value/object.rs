use crate::{
    common::bit_subset64::BitSubset64,
    container::{Container, ContainerRef, Info},
    value::unknown::Unknown,
};

use super::{extension::OBJECT, tag::TagPtr};

pub struct ObjectHeader();

impl Info for ObjectHeader {
    type Item = (Unknown, Unknown);
}

pub type ObjectContainer = Container<ObjectHeader>;

pub type ObjectRef = ContainerRef<ObjectHeader>;

impl TagPtr for ObjectHeader {
    const PTR_SUBSET: BitSubset64 = OBJECT.subset();
}

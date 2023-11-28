use crate::{
    common::bit_subset64::BitSubset64,
    container::{Container, ContainerRef, Info},
};

use super::{extension::STRING, tag::TagPtr};

pub struct StringHeader();

impl Info for StringHeader {
    type Item = u16;
}

pub type StringContainer = Container<StringHeader>;

pub type StringRef = ContainerRef<StringHeader>;

impl TagPtr for StringHeader {
    const PTR_SUBSET: BitSubset64 = STRING.subset();
}

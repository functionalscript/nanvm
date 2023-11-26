use crate::container::{Container, ContainerRef, Info};

pub struct StringHeader();

impl Info for StringHeader {
    type Item = u16;
}

pub type StringContainer = Container<StringHeader>;

pub type StringRef = ContainerRef<StringHeader>;

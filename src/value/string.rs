use crate::container::{Info, ContainerRef};

pub struct StringHeader();

impl Info for StringHeader {
    type Item = u16;
}

pub type StringRef = ContainerRef<StringHeader>;
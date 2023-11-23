use crate::container::{Header, Ref};

pub struct StringHeader();

impl Header for StringHeader {
    type Item = u16;
}

#[repr(transparent)]
pub struct String(Ref<StringHeader>);

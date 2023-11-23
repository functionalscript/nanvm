use crate::container::{Header, Ref};

pub struct StringHeader(usize);

impl Header for StringHeader {
    type Item = u16;
}

#[repr(transparent)]
pub struct String(Ref<StringHeader>);

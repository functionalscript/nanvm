use crate::container::{Info, Ref};

pub struct StringHeader(usize);

impl Info for StringHeader {
    type Item = u16;
}

#[repr(transparent)]
pub struct String(Ref<StringHeader>);

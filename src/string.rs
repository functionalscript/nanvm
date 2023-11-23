use crate::container::{Header, Ref};

pub struct StringHeader(usize);

impl Header for StringHeader {
    type Item = u16;
    fn len(&self) -> usize {
        self.0
    }
}

#[repr(transparent)]
pub struct String(Ref<StringHeader>);

use crate::container::{Info, Ref};

pub struct StringInfo();

impl Info for StringInfo {
    type Item = u16;
}

#[repr(transparent)]
pub struct String(Ref<StringInfo>);

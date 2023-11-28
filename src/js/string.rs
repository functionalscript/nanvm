use crate::{
    common::bit_subset64::BitSubset64,
    container::{Container, Info, Rc},
};

use super::{extension::STRING, tag_rc::TagRc};

pub struct StringHeader();

impl Info for StringHeader {
    type Item = u16;
}

pub type StringContainer = Container<StringHeader>;

pub type StringRc = Rc<StringHeader>;

impl TagRc for StringHeader {
    const RC_SUBSET: BitSubset64 = STRING;
}

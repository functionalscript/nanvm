use crate::{
    allocator::GlobalAllocator,
    common::bit_subset64::BitSubset64,
    container::{Container, Info, Rc},
};

use super::{bitset::STRING, extension_rc::TagRc};

pub struct StringHeader();

impl Info for StringHeader {
    type Item = u16;
    type Allocator = GlobalAllocator;
}

pub type StringContainer = Container<StringHeader>;

pub type StringRc = Rc<StringHeader>;

impl TagRc for StringHeader {
    const RC_SUBSET: BitSubset64 = STRING;
}

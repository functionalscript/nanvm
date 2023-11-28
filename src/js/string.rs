use crate::{
    common::{allocator::GlobalAllocator, bit_subset64::BitSubset64},
    container::{Container, Info, Rc},
};

use super::{bitset::STRING, extension_rc::ExtensionRc};

pub struct StringHeader();

impl Info for StringHeader {
    type Item = u16;
    type Allocator = GlobalAllocator;
}

pub type StringContainer = Container<StringHeader>;

pub type StringRc = Rc<StringHeader>;

impl ExtensionRc for StringHeader {
    const RC_SUBSET: BitSubset64 = STRING;
}

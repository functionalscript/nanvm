use crate::{
    allocator::GlobalAllocator,
    common::bit_subset64::BitSubset64,
    container::{Container, Info, Rc},
    js::any::Any,
};

use super::{bitset::OBJECT, extension_rc::TagRc};

pub struct ObjectHeader();

impl Info for ObjectHeader {
    type Item = (Any, Any);
    type Allocator = GlobalAllocator;
}

pub type ObjectContainer = Container<ObjectHeader>;

pub type ObjectRc = Rc<ObjectHeader>;

impl TagRc for ObjectHeader {
    const RC_SUBSET: BitSubset64 = OBJECT;
}

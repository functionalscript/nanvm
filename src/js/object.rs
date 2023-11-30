use std::marker::PhantomData;

use crate::{
    common::{allocator::GlobalAllocator, bit_subset64::BitSubset64},
    container::{Container, Info, Rc},
    js::any::Any,
};

use super::{bitset::OBJECT, extension_rc::ExtensionRc, string::StringRc};

pub struct ObjectHeader<A = GlobalAllocator>(PhantomData<A>);

impl Default for ObjectHeader {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl Info for ObjectHeader {
    type Item = (StringRc, Any);
    type Allocator = GlobalAllocator;
}

pub type ObjectContainer = Container<ObjectHeader>;

pub type ObjectRc = Rc<ObjectHeader>;

impl ExtensionRc for ObjectHeader {
    const RC_SUBSET: BitSubset64 = OBJECT;
}

use std::marker::PhantomData;

use crate::{
    common::{
        allocator::{Allocator, GlobalAllocator},
        bit_subset64::BitSubset64,
    },
    container::{Container, Info, Rc},
};

use super::{bitset::STRING, extension_rc::ExtensionRc};

pub struct StringHeader<A: Allocator = GlobalAllocator>(PhantomData<A>);

impl Default for StringHeader {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<A: Allocator> Info for StringHeader<A> {
    type Item = u16;
    type Allocator = A;
}

pub type StringContainer = Container<StringHeader>;

pub type StringRc = Rc<StringHeader>;

impl<A: Allocator> ExtensionRc for StringHeader<A> {
    const RC_SUBSET: BitSubset64 = STRING;
}

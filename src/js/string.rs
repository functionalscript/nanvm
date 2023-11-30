use std::marker::PhantomData;

use crate::{
    common::{allocator::Allocator, bit_subset64::BitSubset64},
    container::{Container, Info, Rc},
};

use super::{bitset::STRING, extension_rc::ExtensionRc};

pub struct StringHeader<A>(PhantomData<A>);

impl<A> Default for StringHeader<A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<A: Allocator> Info for StringHeader<A> {
    type Item = u16;
    type Allocator = A;
}

pub type StringContainer<A> = Container<StringHeader<A>>;

pub type StringRc<A> = Rc<StringHeader<A>>;

impl<A: Allocator> ExtensionRc for StringHeader<A> {
    const RC_SUBSET: BitSubset64 = STRING;
}

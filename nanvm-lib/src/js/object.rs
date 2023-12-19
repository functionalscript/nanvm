use core::marker::PhantomData;

use crate::{
    common::{allocator::Allocator, bit_subset64::BitSubset64},
    container::{Container, Info, Rc},
    js::any::{Any, Any2},
    mem::{flexible_array::FlexibleArray, manager::Dealloc},
};

use super::{
    bitset::OBJECT, extension_rc::ExtensionRc, extension_ref::ExtensionRef, string::StringRc,
};

pub struct ObjectHeader<A>(PhantomData<A>);

impl<A> Default for ObjectHeader<A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<A: Allocator> Info for ObjectHeader<A> {
    type Item = (StringRc<A>, Any);
    type Allocator = A;
}

pub type ObjectContainer<A> = Container<ObjectHeader<A>>;

pub type ObjectRc<A> = Rc<ObjectHeader<A>>;

impl<A: Allocator> ExtensionRc for ObjectHeader<A> {
    const RC_SUBSET: BitSubset64 = OBJECT;
}

pub type ObjectHeader2<D> = FlexibleArray<Any2<D>>;

impl<D: Dealloc> ExtensionRef for ObjectHeader2<D> {
    const REF_SUBSET: BitSubset64 = OBJECT;
}

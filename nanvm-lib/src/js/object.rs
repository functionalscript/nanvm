use core::marker::PhantomData;

use crate::{
    common::{allocator::Allocator, bit_subset64::BitSubset64},
    container::{Container, Info, Rc},
    js::any::Any2,
    mem::{flexible_array::FlexibleArray, manager::Dealloc},
};

use super::{bitset::OBJECT, extension_ref::ExtensionRef};

pub type ObjectHeader2<D> = FlexibleArray<Any2<D>>;

impl<D: Dealloc> ExtensionRef for ObjectHeader2<D> {
    const REF_SUBSET: BitSubset64 = OBJECT;
}

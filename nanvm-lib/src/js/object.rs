use crate::{
    common::bit_subset64::BitSubset64,
    js::any::Any,
    mem::{flexible_array::FlexibleArray, manager::Dealloc},
};

use super::{bitset::OBJECT, extension_ref::ExtensionRef};

pub type ObjectHeader<D> = FlexibleArray<Any<D>>;

impl<D: Dealloc> ExtensionRef for ObjectHeader<D> {
    const REF_SUBSET: BitSubset64 = OBJECT;
}

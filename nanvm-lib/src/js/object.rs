use crate::{
    common::bit_subset64::BitSubset64,
    js::any::Any,
    mem::{block::Block, flexible_array::FlexibleArray, manager::Dealloc},
};

use super::{bitset::OBJECT, ref_cast::RefCast};

pub type ObjectHeader<D> = FlexibleArray<Any<D>>;

impl<D: Dealloc> RefCast<D> for ObjectHeader<D> {
    const REF_SUBSET: BitSubset64<*const Block<Self, D>> = OBJECT.cast();
}

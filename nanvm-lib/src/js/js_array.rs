use crate::{
    common::bit_subset64::BitSubset64,
    mem::{block::Block, flexible_array::FlexibleArray, manager::Dealloc, ref_::Ref},
};

use super::{any::Any, bitset::ARRAY, ref_cast::RefCast};

pub type JsArray<D> = FlexibleArray<Any<D>>;

pub type JsArrayRef<D> = Ref<JsArray<D>, D>;

impl<D: Dealloc> RefCast<D> for JsArray<D> {
    const REF_SUBSET: BitSubset64<*const Block<Self, D>> = ARRAY.cast();
}

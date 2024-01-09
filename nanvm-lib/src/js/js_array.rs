use crate::{mem::{flexible_array::FlexibleArray, ref_::Ref, manager::Dealloc, block::Block}, common::bit_subset64::BitSubset64};

use super::{any::Any, ref_cast::RefCast, bitset::ARRAY};

pub type JsArray<D> = FlexibleArray<Any<D>>;

pub type JsArrayRef<D> = Ref<JsArray<D>, D>;

impl<D: Dealloc> RefCast<D> for JsArray<D> {
    const REF_SUBSET: BitSubset64<*const Block<Self, D>> = ARRAY.cast();
}

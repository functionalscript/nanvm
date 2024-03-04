use crate::{
    common::bit_subset64::BitSubset64,
    mem::{
        block::Block,
        flexible_array::FlexibleArray,
        manager::{Dealloc, Manager},
        mut_ref::MutRef,
        ref_::Ref,
    },
};

use super::{any::Any, bitset::ARRAY, ref_cast::RefCast};

pub type JsArray<D> = FlexibleArray<Any<D>>;

pub type JsArrayRef<D> = Ref<JsArray<D>, D>;

pub type JsArrayMutRef<D> = MutRef<JsArray<D>, D>;

impl<D: Dealloc> RefCast<D> for JsArray<D> {
    const REF_SUBSET: BitSubset64<*const Block<Self, D>> = ARRAY.cast();
}

pub fn new_array<M: Manager, I: ExactSizeIterator<Item = Any<M::Dealloc>>>(
    m: M,
    i: impl IntoIterator<IntoIter = I>,
) -> JsArrayMutRef<M::Dealloc> {
    m.flexible_array_new(i.into_iter())
}

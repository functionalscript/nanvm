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

use super::{bitset::STRING, ref_cast::RefCast};

pub type JsString = FlexibleArray<u16>;

pub type JsStringRef<D> = Ref<JsString, D>;

pub type JsStringMutRef<D> = MutRef<JsString, D>;

impl<D: Dealloc> RefCast<D> for JsString {
    const REF_SUBSET: BitSubset64<*const Block<JsString, D>> = STRING.cast();
}

pub fn new_string<M: Manager>(
    m: M,
    i: impl ExactSizeIterator<Item = u16>,
) -> JsStringMutRef<M::Dealloc> {
    m.flexible_array_new(i)
}

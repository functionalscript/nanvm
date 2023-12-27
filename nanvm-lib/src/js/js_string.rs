use crate::{
    common::bit_subset64::BitSubset64,
    mem::{block::Block, flexible_array::FlexibleArray, manager::Dealloc, ref_::Ref},
};

use super::{bitset::STRING, ref_cast::RefCast};

pub type JsString = FlexibleArray<u16>;

pub type JsStringRef<D> = Ref<JsString, D>;

impl<D: Dealloc> RefCast<D> for JsString {
    const REF_SUBSET: BitSubset64<*const Block<JsString, D>> = STRING.cast();
}

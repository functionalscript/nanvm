use crate::{
    common::bit_subset64::BitSubset64,
    js::any::Any,
    mem::{block::Block, flexible_array::FlexibleArray, manager::Dealloc},
};

use super::{bitset::OBJECT, js_string::JsStringRef, ref_cast::RefCast};

pub type JsObject<D> = FlexibleArray<Any<D>>;

impl<D: Dealloc> RefCast<D> for JsObject<D> {
    const REF_SUBSET: BitSubset64<*const Block<Self, D>> = OBJECT.cast();
}

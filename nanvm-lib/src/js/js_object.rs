use crate::{
    common::bit_subset64::BitSubset64,
    js::any::Any,
    mem::{block::Block, flexible_array::FlexibleArray, manager::Dealloc, ref_::Ref},
};

use super::{bitset::OBJECT, js_string::JsStringRef, ref_cast::RefCast};

pub type JsObject<D> = FlexibleArray<(JsStringRef<D>, Any<D>)>;

pub type JsObjectRef<D> = Ref<JsObject<D>, D>;

impl<D: Dealloc> RefCast<D> for JsObject<D> {
    const REF_SUBSET: BitSubset64<*const Block<Self, D>> = OBJECT.cast();
}

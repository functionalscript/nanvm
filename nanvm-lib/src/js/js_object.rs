use crate::{
    common::bit_subset64::BitSubset64,
    js::any::Any,
    mem::{
        block::Block,
        flexible_array::FlexibleArray,
        manager::{Dealloc, Manager},
        mut_ref::MutRef,
        ref_::Ref,
    },
};

use super::{bitset::OBJECT, js_string::JsStringRef, ref_cast::RefCast};

pub type Property<D> = (JsStringRef<D>, Any<D>);

pub type JsObject<D> = FlexibleArray<Property<D>>;

pub type JsObjectRef<D> = Ref<JsObject<D>, D>;

pub type JsObjectMutRef<D> = MutRef<JsObject<D>, D>;

impl<D: Dealloc> RefCast<D> for JsObject<D> {
    const REF_SUBSET: BitSubset64<*const Block<Self, D>> = OBJECT.cast();
}

pub fn new_object<M: Manager>(
    m: M,
    i: impl ExactSizeIterator<Item = Property<M::Dealloc>>,
) -> JsObjectMutRef<M::Dealloc> {
    m.flexible_array_new(i)
}

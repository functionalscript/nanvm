use core::marker::PhantomData;

use crate::mem::{
    block_header::BlockHeader, global::Global, manager::Dealloc, optional_block::OptionalBlock,
};

use super::{
    bitset::{ref_type, REF, REF_SUBSET_SUPERPOSITION},
    js_array::JsArray,
    js_object::JsObject,
    js_string::JsString,
};

#[repr(transparent)]
pub struct AnyInternal<D: Dealloc = Global>(pub u64, PhantomData<D>);

impl<D: Dealloc> Clone for AnyInternal<D> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<D: Dealloc> Copy for AnyInternal<D> {}

impl<D: Dealloc> AnyInternal<D> {
    pub const fn new(v: u64) -> Self {
        Self(v, PhantomData)
    }
}

impl<D: Dealloc> OptionalBlock for AnyInternal<D> {
    type BlockHeader = D::BlockHeader;
    #[inline(always)]
    fn is_ref(self) -> bool {
        REF.has(self.0)
    }
    #[inline(always)]
    unsafe fn try_get_block_header(self) -> Option<*const Self::BlockHeader> {
        if self.is_ref() {
            Some((self.0 & REF_SUBSET_SUPERPOSITION) as _)
        } else {
            None
        }
    }
    #[inline(always)]
    unsafe fn delete(self, block_header: *mut Self::BlockHeader) {
        let p = &mut *block_header;
        match ref_type(self.0) {
            REF_TYPE_STRING => p.block::<JsString, D>().delete(),
            REF_TYPE_OBJECT => p.block::<JsObject<D>, D>().delete(),
            REF_TYPE_ARRAY => p.block::<JsArray<D>, D>().delete(),
            _ => unreachable!(),
        }
    }
}

use core::{
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use crate::{
    js::bitset,
    mem::{
        block_header::BlockHeader, global::Global, manager::Dealloc, optional_block::OptionalBlock,
    },
};

use super::{
    bitset::{ref_type, REF, REF_SUBSET_SUPERPOSITION},
    js_array::JsArray,
    js_object::JsObject,
    js_string::JsString,
};

#[repr(transparent)]
#[derive(Debug)]
pub struct AnyInternal<D: Dealloc = Global>(pub u64, PhantomData<D>);

impl<D: Dealloc> PartialEq for AnyInternal<D> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<D: Dealloc> Hash for AnyInternal<D> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<D: Dealloc> Clone for AnyInternal<D> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
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
            bitset::REF_TYPE_STRING => p.block::<JsString, D>().delete(),
            bitset::REF_TYPE_OBJECT => p.block::<JsObject<D>, D>().delete(),
            bitset::REF_TYPE_ARRAY => p.block::<JsArray<D>, D>().delete(),
            _ => unreachable!(),
        }
    }
}

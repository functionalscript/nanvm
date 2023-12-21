use core::marker::PhantomData;

use crate::mem::{
    block_header::BlockHeader, global::Global, manager::Dealloc, optional_block::OptionalBlock,
};

use super::{
    bitset::{REF, REF_SUBSET_SUPERPOSITION, STRING},
    object::ObjectHeader,
    string::StringHeader,
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
        if STRING.has(self.0) {
            (*block_header).block::<StringHeader, D>().delete();
        } else {
            (*block_header).block::<ObjectHeader<D>, D>().delete();
        }
    }
}

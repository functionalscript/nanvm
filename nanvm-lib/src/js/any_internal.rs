use core::marker::PhantomData;

use crate::{
    common::allocator::GlobalAllocator,
    container::{Base, OptionalBase},
    mem::{
        block_header::BlockHeader, global::Global, manager::Dealloc, optional_block::OptionalBlock,
    },
};

use super::{
    bitset::{RC, RC_SUBSET_SUPERPOSITION, STRING},
    extension_rc::ExtensionRc,
    object::{Object2, ObjectHeader},
    string::{String2, StringHeader},
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

impl OptionalBase for AnyInternal {
    unsafe fn get_base(&self) -> Option<*mut Base> {
        let v = self.0;
        if !RC.has(v) {
            return None;
        }
        Some((v & RC_SUBSET_SUPERPOSITION) as *mut Base)
    }
    unsafe fn delete(&self, base: *mut Base) {
        if STRING.has(self.0) {
            StringHeader::<GlobalAllocator>::delete(base);
        } else {
            ObjectHeader::<GlobalAllocator>::delete(base);
        }
    }
}

impl<D: Dealloc> OptionalBlock for AnyInternal<D> {
    type BlockHeader = D::BlockHeader;
    #[inline(always)]
    fn try_get_block_header(self) -> Option<*const Self::BlockHeader> {
        let v = self.0;
        if !RC.has(v) {
            return None;
        }
        Some((v & RC_SUBSET_SUPERPOSITION) as _)
    }
    #[inline(always)]
    unsafe fn delete(self, block_header: *mut Self::BlockHeader) {
        if STRING.has(self.0) {
            (*block_header).block::<String2, D>().delete();
        } else {
            (*block_header).block::<Object2, D>().delete();
        }
    }
}

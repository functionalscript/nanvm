use core::marker::PhantomData;

use crate::{
    common::allocator::GlobalAllocator,
    container::{Base, OptionalBase},
    mem::{global::Global, manager::Dealloc},
};

use super::{
    bitset::{RC, RC_SUBSET_SUPERPOSITION, STRING},
    extension_rc::ExtensionRc,
    object::ObjectHeader,
    string::StringHeader,
};

#[repr(transparent)]
#[derive(Copy)]
pub struct AnyInternal<D: Dealloc = Global>(pub u64, PhantomData<D>);

impl<D: Dealloc> Clone for AnyInternal<D> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

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

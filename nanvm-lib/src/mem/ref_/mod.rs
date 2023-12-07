pub mod update;

use core::ops::Deref;

use self::update::RefUpdate;

use super::{
    block::{header::BlockHeader, Block},
    manager::Manager,
    object::Object,
};

/// A reference to an object allocated by a memory manager.
#[repr(transparent)]
pub struct Ref<T: Object, M: Manager> { p: *mut Block<M::BlockHeader, T> }

impl<T: Object, M: Manager> Deref for Ref<T, M> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { (*self.p).object() }
    }
}

impl<T: Object, M: Manager> Ref<T, M> {
    #[inline(always)]
    pub unsafe fn new(p: *mut Block<M::BlockHeader, T>) -> Self {
        Self { p }
    }
}

impl<T: Object, M: Manager> Clone for Ref<T, M> {
    #[inline(always)]
    fn clone(&self) -> Self {
        let p = self.p;
        unsafe { (*p).header.ref_update(RefUpdate::AddRef) };
        Self { p }
    }
}

impl<T: Object, M: Manager> Drop for Ref<T, M> {
    fn drop(&mut self) {
        unsafe {
            let p = &mut *self.0;
            if p.header.ref_update(RefUpdate::Release) == 0 {
                p.delete();
            }
        }
    }
}

// TODO: tests

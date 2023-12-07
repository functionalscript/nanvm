pub mod update;

use core::{mem::forget, ops::Deref};

use self::update::RefUpdate;

use super::{
    block::{header::BlockHeader, Block},
    manager::Manager,
    mut_ref::MutRef,
    object::Object,
};

/// A reference to an object allocated by a memory manager.
#[repr(transparent)]
pub struct Ref<T: Object, M: Manager> {
    p: *mut Block<M::BlockHeader, T>,
}

impl<T: Object, M: Manager> Ref<T, M> {
    #[inline(always)]
    pub unsafe fn new(p: *mut Block<M::BlockHeader, T>) -> Self {
        Self { p }
    }
    #[inline(always)]
    unsafe fn ref_update(&self, i: RefUpdate) -> isize {
        (*self.p).header.ref_update(i)
    }
    pub fn try_to_mut_ref(mut self) -> Result<MutRef<T, M>, Self> {
        unsafe {
            if self.ref_update(RefUpdate::Read) == 1 {
                let result = MutRef::new(self.p);
                forget(self);
                Ok(result)
            } else {
                Err(self)
            }
        }
    }
}

impl<T: Object, M: Manager> Clone for Ref<T, M> {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            self.ref_update(RefUpdate::AddRef);
            Self { p: self.p }
        }
    }
}

impl<T: Object, M: Manager> Drop for Ref<T, M> {
    fn drop(&mut self) {
        unsafe {
            if self.ref_update(RefUpdate::Release) == 1 {
                (*self.p).delete();
            }
        }
    }
}

impl<T: Object, M: Manager> Deref for Ref<T, M> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { (*self.p).object() }
    }
}

// TODO: tests

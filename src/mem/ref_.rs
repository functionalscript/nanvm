use core::marker::PhantomData;

use super::{block_header::BlockHeader, object::Object, rc_update::RcUpdate, Manager};

/// A reference to an object allocated by a memory manager.
#[repr(transparent)]
pub struct Ref<T: Object, M: Manager>(*mut M::BlockHeader, PhantomData<T>);

impl<T: Object, M: Manager> Ref<T, M> {
    pub unsafe fn new(v: *mut M::BlockHeader) -> Self {
        Self(v, PhantomData)
    }
}

impl<T: Object, M: Manager> Clone for Ref<T, M> {
    fn clone(&self) -> Self {
        let v = self.0;
        unsafe { (*v).rc_update(RcUpdate::AddRef) };
        Self(v, PhantomData)
    }
}

impl<T: Object, M: Manager> Drop for Ref<T, M> {
    fn drop(&mut self) {
        unsafe {
            let p = &mut *self.0;
            if p.rc_update(RcUpdate::Release) == 0 {
                p.delete::<T>();
            }
        }
    }
}

pub mod update;

use core::marker::PhantomData;

use self::update::RefUpdate;

use super::{block_header::BlockHeader, object::Object, Manager};

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
        unsafe { (*v).ref_update(RefUpdate::AddRef) };
        Self(v, PhantomData)
    }
}

impl<T: Object, M: Manager> Drop for Ref<T, M> {
    fn drop(&mut self) {
        unsafe {
            let p = &mut *self.0;
            if p.ref_update(RefUpdate::Release) == 0 {
                p.delete::<T>();
            }
        }
    }
}

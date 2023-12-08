use core::{
    mem::forget,
    ops::{Deref, DerefMut},
};

use crate::mem::{
    block::{header::BlockHeader, Block},
    manager::Manager,
    object::Object,
    ref_::update::RefUpdate,
};

use super::{manager::Dealloc, ref_::Ref};

/// A reference to a mutable object allocated by a memory manager.
#[repr(transparent)]
#[derive(Debug)]
pub struct MutRef<T: Object, D: Dealloc>(*mut Block<D, T>);

impl<T: Object, D: Dealloc> MutRef<T, D> {
    #[inline(always)]
    pub unsafe fn new(v: *mut Block<D, T>) -> Self {
        Self(v)
    }
    #[inline(always)]
    pub fn to_ref(self) -> Ref<T, D> {
        let result = unsafe { Ref::new(self.0) };
        forget(self);
        result
    }
}

impl<T: Object, D: Dealloc> Drop for MutRef<T, D> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { (&mut *self.0).delete() }
    }
}

impl<T: Object, D: Dealloc> Deref for MutRef<T, D> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { (*self.0).object() }
    }
}

impl<T: Object, D: Dealloc> DerefMut for MutRef<T, D> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { (*self.0).object_mut() }
    }
}

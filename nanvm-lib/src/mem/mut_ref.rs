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

use super::ref_::Ref;

/// A reference to a mutable object allocated by a memory manager.
#[repr(transparent)]
#[derive(Debug)]
pub struct MutRef<T: Object, M: Manager>(*mut Block<M, T>);

impl<T: Object, M: Manager> MutRef<T, M> {
    #[inline(always)]
    pub unsafe fn new(v: *mut Block<M, T>) -> Self {
        let result = Self(v);
        result.valid_assert();
        result
    }
    #[inline(always)]
    fn valid_assert(&self) {
        unsafe { assert_eq!((*self.0).header.ref_update(RefUpdate::Read), 0) };
    }
    #[inline(always)]
    pub fn to_ref(self) -> Ref<T, M> {
        self.valid_assert();
        let result = unsafe { Ref::new(self.0) };
        forget(self);
        result
    }
}

impl<T: Object, M: Manager> Drop for MutRef<T, M> {
    #[inline(always)]
    fn drop(&mut self) {
        self.valid_assert();
        unsafe { (&mut *self.0).delete() }
    }
}

impl<T: Object, M: Manager> Deref for MutRef<T, M> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { (*self.0).object() }
    }
}

impl<T: Object, M: Manager> DerefMut for MutRef<T, M> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { (*self.0).object_mut() }
    }
}

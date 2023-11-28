use std::{mem::forget, ptr::read};

use super::{OptionalBase, Update};

#[repr(transparent)]
pub struct OptionalRc<T: OptionalBase>(T);

impl<T: OptionalBase> OptionalRc<T> {
    #[inline(always)]
    pub unsafe fn from_internal(t: T) -> Self {
        Self(t)
    }
    #[inline(always)]
    pub unsafe fn internal(&self) -> &T {
        &self.0
    }
    #[inline(always)]
    pub unsafe fn move_to_internal(mut self) -> T {
        let result = read(&mut self.0);
        forget(self);
        result
    }
}

impl<T: OptionalBase> Clone for OptionalRc<T> {
    fn clone(&self) -> Self {
        unsafe {
            let result = Self::from_internal(self.0.clone());
            if let Some(base) = result.0.get_base() {
                (&mut *base).update(Update::AddRef);
            }
            result
        }
    }
}

impl<T: OptionalBase> Drop for OptionalRc<T> {
    fn drop(&mut self) {
        unsafe {
            if let Some(base) = self.0.get_base() {
                if (&mut *base).update(Update::Release) == 0 {
                    self.0.dealloc(base)
                }
            }
        }
    }
}

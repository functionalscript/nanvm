use std::{mem::forget, ptr::read};

use super::{OptionalBase, Update};

#[repr(transparent)]
pub struct OptionalRc<T: OptionalBase>(T);

impl<T: OptionalBase> OptionalRc<T> {
    #[inline(always)]
    pub unsafe fn from_optional_base(t: T) -> Self {
        Self(t)
    }
    #[inline(always)]
    pub unsafe fn optional_base(&self) -> &T {
        &self.0
    }
    #[inline(always)]
    pub unsafe fn move_to_optional_base(mut self) -> T {
        let result = read(&mut self.0);
        forget(self);
        result
    }
}

impl<T: OptionalBase> Clone for OptionalRc<T> {
    fn clone(&self) -> Self {
        unsafe {
            let result = Self::from_optional_base(self.0.clone());
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
                    self.0.delete(base)
                }
            }
        }
    }
}

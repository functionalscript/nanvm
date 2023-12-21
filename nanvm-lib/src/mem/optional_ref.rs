use core::mem::forget;

use crate::common::ref_mut::RefMut;

use super::{optional_block::OptionalBlock, ref_counter_update::RefCounterUpdate};

#[derive(Debug)]
#[repr(transparent)]
pub struct OptionalRef<T: OptionalBlock> {
    value: T,
}

impl<T: OptionalBlock> OptionalRef<T> {
    #[inline(always)]
    pub const unsafe fn from_internal(value: T) -> Self {
        Self { value }
    }
    #[inline(always)]
    pub const unsafe fn internal(&self) -> T {
        self.value
    }
    #[inline(always)]
    pub fn is_ref(&self) -> bool {
        self.value.is_ref()
    }
    #[inline(always)]
    pub unsafe fn move_to_internal(mut self) -> T {
        let result = self.value.to_mut_ptr().read();
        forget(self);
        result
    }
}

impl<T: OptionalBlock> Clone for OptionalRef<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe { self.value.ref_counter_update(RefCounterUpdate::AddRef) };
        Self { value: self.value }
    }
}

impl<T: OptionalBlock> Drop for OptionalRef<T> {
    fn drop(&mut self) {
        unsafe {
            if let Some(header) = self.value.ref_counter_update(RefCounterUpdate::Release) {
                self.value.delete(header);
            }
        }
    }
}

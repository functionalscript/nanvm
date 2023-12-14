use super::{optional_block_ptr::OptionalBlockPtr, ref_counter_update::RefCounterUpdate};

#[derive(Debug)]
#[repr(transparent)]
pub struct OptionalRef<T: OptionalBlockPtr> {
    value: T,
}

impl<T: OptionalBlockPtr> OptionalRef<T> {
    #[inline(always)]
    pub const unsafe fn new(value: T) -> Self {
        Self { value }
    }
    #[inline(always)]
    pub const unsafe fn internal(&self) -> T {
        self.value
    }
}

impl<T: OptionalBlockPtr> Clone for OptionalRef<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe { self.value.ref_counter_update(RefCounterUpdate::AddRef) };
        Self { value: self.value }
    }
}

impl<T: OptionalBlockPtr> Drop for OptionalRef<T> {
    fn drop(&mut self) {
        unsafe {
            if let Some(header) = self.value.ref_counter_update(RefCounterUpdate::Release) {
                self.value.delete(header);
            }
        }
    }
}

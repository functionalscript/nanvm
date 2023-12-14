use super::{optional_ptr::OptionalPtr, ref_counter_update::RefCounterUpdate};

#[derive(Debug)]
#[repr(transparent)]
pub struct OptionalRef<T: OptionalPtr> {
    value: T,
}

impl<T: OptionalPtr> OptionalRef<T> {
    #[inline(always)]
    pub const unsafe fn new(value: T) -> Self {
        Self { value }
    }
    #[inline(always)]
    pub const unsafe fn internal(&self) -> T {
        self.value
    }
}

impl<T: OptionalPtr> Clone for OptionalRef<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe { self.value.ref_counter_update(RefCounterUpdate::AddRef) };
        Self { value: self.value }
    }
}

impl<T: OptionalPtr> Drop for OptionalRef<T> {
    fn drop(&mut self) {
        unsafe {
            if let Some(header) = self.value.ref_counter_update(RefCounterUpdate::Release) {
                self.value.delete(header);
            }
        }
    }
}

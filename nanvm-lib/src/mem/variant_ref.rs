use super::{ref_counter_update::RefCounterUpdate, variant::Variant};

#[derive(Debug)]
#[repr(transparent)]
pub struct VariantRef<T: Variant> {
    value: T,
}

impl<T: Variant> VariantRef<T> {
    #[inline(always)]
    pub const unsafe fn new(value: T) -> Self {
        Self { value }
    }
    #[inline(always)]
    pub const unsafe fn internal(&self) -> T {
        self.value
    }
}

impl<T: Variant> Clone for VariantRef<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe { self.value.ref_counter_update(RefCounterUpdate::AddRef) };
        Self { value: self.value }
    }
}

impl<T: Variant> Drop for VariantRef<T> {
    fn drop(&mut self) {
        unsafe {
            if let Some(header) = self.value.ref_counter_update(RefCounterUpdate::Release) {
                self.value.delete(header);
            }
        }
    }
}

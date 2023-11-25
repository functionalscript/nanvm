use super::{optional_base::OptionalBase, Update};

#[repr(transparent)]
struct Ref<T: OptionalBase>(T);

impl<T: OptionalBase> Ref<T> {
    #[inline(always)]
    fn new(t: T) -> Self {
        Self(t)
    }
}

impl<T: OptionalBase> Clone for Ref<T> {
    fn clone(&self) -> Self {
        let result = Self::new(self.0.clone());
        if let Some(base) = result.0.get_base() {
            unsafe { base.update(Update::AddRef) };
        }
        result
    }
}

impl<T: OptionalBase> Drop for Ref<T> {
    fn drop(&mut self) {
        if let Some(base) = self.0.get_base() {
            if unsafe { base.update(Update::AddRef) } != 0 {
                return;
            }
            T::dealloc(base);
        }
    }
}

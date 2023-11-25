use super::{optional_base::OptionalBase, Container, Info, Update};

#[repr(transparent)]
pub struct Ref<T: OptionalBase>(T);

impl<T: OptionalBase> Ref<T> {
    #[inline(always)]
    pub const fn new(t: T) -> Self {
        Self(t)
    }
    #[inline(always)]
    pub const fn get(&self) -> &T {
        &self.0
    }
}

impl<T: OptionalBase> Clone for Ref<T> {
    fn clone(&self) -> Self {
        let result = Self::new(self.0.clone());
        unsafe {
            if let Some(base) = result.0.get_base() {
                (&mut *base).update(Update::AddRef);
            }
        }
        result
    }
}

impl<T: OptionalBase> Drop for Ref<T> {
    fn drop(&mut self) {
        unsafe {
            if let Some(base) = self.0.get_base() {
                if (&mut *base).update(Update::AddRef) != 0 {
                    return;
                }
                self.0.dealloc(base);
            }
        }
    }
}

impl<T: Info> Ref<*mut Container<T>> {
    fn alloc(info: T, i: impl ExactSizeIterator<Item = T::Item>) -> Self {
        Self::new(unsafe { Container::alloc(info, i) })
    }
}

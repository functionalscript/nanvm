use std::mem::forget;

use super::{optional_base::OptionalBase, Container, Info, Update};

#[repr(transparent)]
pub struct Ref<T: OptionalBase>(T);

impl<T: OptionalBase> Ref<T> {
    #[inline(always)]
    pub const fn from_raw(t: T) -> Self {
        Self(t)
    }
    #[inline(always)]
    pub const fn get(&self) -> &T {
        &self.0
    }
    #[inline(always)]
    pub const fn move_to_raw(self) -> T {
        let result = self.0;
        forget(self);
        result
    }
}

impl<T: OptionalBase> Clone for Ref<T> {
    fn clone(&self) -> Self {
        let result = Self::from_raw(self.0.clone());
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

pub type ContainerRef<T> = Ref<*mut Container<T>>;

impl<T: Info> ContainerRef<T> {
    pub fn alloc(info: T, i: impl ExactSizeIterator<Item = T::Item>) -> Self {
        Self::from_raw(unsafe { Container::alloc(info, i) })
    }
    pub fn get_items_mut(&self) -> &mut [T::Item] {
        unsafe { (*self.0).get_items_mut() }
    }
}

use std::ops::Deref;

use crate::container::{Container, CLONE, DROP};

struct Ref<T>(*mut Container<T>);

impl<T> Ref<T> {
    pub fn new(p: &mut Container<T>) -> Self {
        unsafe { Container::update::<CLONE>(p) };
        Self(p)
    }
}

impl<T> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Self::new(unsafe { &mut *self.0 })
    }
}

impl<T> Drop for Ref<T> {
    fn drop(&mut self) {
        unsafe {
            Container::update::<DROP>(self.0);
        }
    }
}

impl<T> Deref for Ref<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.0).value }
    }
}

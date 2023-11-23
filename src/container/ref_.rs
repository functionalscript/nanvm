use std::ops::Deref;

use super::{Container, Header};

pub struct Ref<T: Header>(*mut Container<T>);

impl<T: Header> Ref<T> {
    pub fn new(p: &mut Container<T>) -> Self {
        unsafe { Container::add_ref(p) };
        Self(p)
    }
}

impl<T: Header> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Self::new(unsafe { &mut *self.0 })
    }
}

impl<T: Header> Drop for Ref<T> {
    fn drop(&mut self) {
        unsafe {
            Container::release(self.0);
        }
    }
}

impl<T: Header> Deref for Ref<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.0).value }
    }
}

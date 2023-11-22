use std::ops::Deref;

use super::{Container, Info, CLONE, DROP};

pub struct Ref<T: Info>(*mut Container<T>);

impl<T: Info> Ref<T> {
    pub fn new(p: &mut Container<T>) -> Self {
        unsafe { Container::update::<CLONE>(p) };
        Self(p)
    }
}

impl<T: Info> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Self::new(unsafe { &mut *self.0 })
    }
}

impl<T: Info> Drop for Ref<T> {
    fn drop(&mut self) {
        unsafe {
            Container::update::<DROP>(self.0);
        }
    }
}

impl<T: Info> Deref for Ref<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.0).value }
    }
}

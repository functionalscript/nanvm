use std::ops::Deref;

use crate::{
    containable::Containable,
    container::{Container, CLONE, DROP},
};

pub struct Ref<T: Containable>(*mut Container<T>);

impl<T: Containable> Ref<T> {
    pub fn new(p: &mut Container<T>) -> Self {
        unsafe { Container::update::<CLONE>(p) };
        Self(p)
    }
}

impl<T: Containable> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Self::new(unsafe { &mut *self.0 })
    }
}

impl<T: Containable> Drop for Ref<T> {
    fn drop(&mut self) {
        unsafe {
            Container::update::<DROP>(self.0);
        }
    }
}

impl<T: Containable> Deref for Ref<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.0).value }
    }
}

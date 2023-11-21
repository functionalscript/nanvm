use std::ops::Deref;

use crate::container::{Clean, Container, DROP, CLONE};

struct Ref<T: Clean>(*mut Container<T>);

impl<T: Clean> Ref<T> {
    pub unsafe fn new(p: *mut Container<T>) -> Self {
        Container::update::<CLONE>(p);
        Self(p)
    }
}

impl<T: Clean> Clone for Ref<T> {
    fn clone(&self) -> Self {
        unsafe { Self::new(self.0) }
    }
}

impl<T: Clean> Drop for Ref<T> {
    fn drop(&mut self) {
        unsafe {
            Container::update::<DROP>(self.0);
        }
    }
}

impl<T: Clean> Deref for Ref<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.0).value }
    }
}
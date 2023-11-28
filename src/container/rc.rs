use std::{mem::forget, ptr::read};

use super::{optional_base::OptionalBase, optional_rc::OptionalRc, Container, Info, Update};

pub type Rc<T> = OptionalRc<*mut Container<T>>;

impl<T: Info> Rc<T> {
    pub fn alloc(info: T, i: impl ExactSizeIterator<Item = T::Item>) -> Self {
        unsafe { Self::from_internal(Container::new(info, i)) }
    }
    pub fn get_items_mut(&self) -> &mut [T::Item] {
        unsafe { (**self.internal()).get_items_mut() }
    }
}

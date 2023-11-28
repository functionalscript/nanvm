use super::{optional_rc::OptionalRc, Container, Info};

pub type Rc<T> = OptionalRc<*mut Container<T>>;

impl<T: Info> Rc<T> {
    pub fn alloc(
        allocator: T::Allocator,
        info: T,
        i: impl ExactSizeIterator<Item = T::Item>,
    ) -> Self {
        unsafe { Self::from_internal(Container::new(allocator, info, i)) }
    }
    pub fn get_items_mut(&self) -> &mut [T::Item] {
        unsafe { (**self.internal()).get_items_mut() }
    }
}

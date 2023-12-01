use super::{Base, Container, Info};

pub trait OptionalBase: Clone {
    unsafe fn get_base(&self) -> Option<*mut Base>;
    unsafe fn delete(&self, base: *mut Base);
}

impl<T: Info> OptionalBase for *mut Container<T> {
    #[inline(always)]
    unsafe fn get_base(&self) -> Option<*mut Base> {
        Some(*self as *mut Base)
    }
    #[inline(always)]
    unsafe fn delete(&self, _: *mut Base) {
        Container::delete(*self);
    }
}

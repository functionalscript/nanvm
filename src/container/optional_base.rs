use super::{Base, Container, Info};

pub trait OptionalBase: Copy {
    unsafe fn get_base(&self) -> Option<*mut Base>;
    unsafe fn dealloc(&self, base: *mut Base);
}

impl<T: Info> OptionalBase for *mut Container<T> {
    #[inline(always)]
    unsafe fn get_base(&self) -> Option<*mut Base> {
        Some(*self as *mut Base)
    }
    #[inline(always)]
    unsafe fn dealloc(&self, _: *mut Base) {
        Container::dealloc(*self);
    }
}

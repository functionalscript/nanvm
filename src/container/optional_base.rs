use super::Base;

pub trait OptionalBase: Clone {
    unsafe fn get_base(&self) -> Option<*mut Base>;
    unsafe fn dealloc(&self, base: *mut Base);
}

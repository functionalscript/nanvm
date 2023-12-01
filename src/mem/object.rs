use core::{
    mem::{align_of, size_of},
    ptr::drop_in_place,
};

/// Object properties
pub trait Object {
    const ALIGN: usize;
    fn size(&self) -> usize;
    unsafe fn drop_in_place(&mut self);
}

impl<T> Object for T {
    const ALIGN: usize = align_of::<T>();
    fn size(&self) -> usize {
        size_of::<T>()
    }
    unsafe fn drop_in_place(&mut self) {
        drop_in_place(self)
    }
}

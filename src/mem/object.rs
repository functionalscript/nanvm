use core::{
    mem::{align_of, size_of},
    ptr::drop_in_place,
};

/// Object properties
pub trait Object: Sized {
    const ALIGN: usize = align_of::<Self>();
    fn size(&self) -> usize {
        size_of::<Self>()
    }
    unsafe fn drop_in_place(&mut self) {
        drop_in_place(self)
    }
}

use core::{
    mem::{align_of, size_of},
    ptr::drop_in_place,
};

trait ObjectHolder {
    type Object: Object;
    fn object(&self) -> &Self::Object;
}

/// Object properties
pub trait Object: Sized {
    const OBJECT_ALIGN: usize = align_of::<Self>();
    fn object_size(&self) -> usize {
        size_of::<Self>()
    }
    unsafe fn object_drop_in_place(&mut self) {
        drop_in_place(self)
    }
}

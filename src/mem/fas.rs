use core::{
    mem::{align_of, size_of},
    ptr::drop_in_place,
    slice::from_raw_parts_mut,
};

use crate::mem::field_layout::FieldLayout;

use super::Object;

pub trait FasHeader: Sized {
    // required
    type Item;
    fn len(&self) -> usize;
    // optional
    const LAYOUT: FieldLayout<Self, Self::Item> = FieldLayout::align_to(align_of::<Self::Item>());
    fn get_items_mut(&mut self) -> &mut [Self::Item] {
        unsafe { from_raw_parts_mut(Self::LAYOUT.to_adjacent(self), self.len()) }
    }
}

#[repr(transparent)]
pub struct Fas<T: FasHeader>(pub T);

impl<T: FasHeader> Object for Fas<T> {
    const ALIGN: usize = T::LAYOUT.align;
    fn size(&self) -> usize {
        T::LAYOUT.size + self.0.len() * size_of::<T::Item>()
    }
    unsafe fn drop_in_place(&mut self) {
        drop_in_place(self.0.get_items_mut());
        drop_in_place(self);
    }
}

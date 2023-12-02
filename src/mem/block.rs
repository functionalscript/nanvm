use core::{alloc::Layout, marker::PhantomData};

use crate::common::ref_mut::RefMut;

use super::{field_layout::FieldLayout, object::Object, ref_::update::RefUpdate};

#[repr(transparent)]
pub struct Block<BH: BlockHeader, T: Object>(pub BH, PhantomData<T>);

impl<BH: BlockHeader, T: Object> Block<BH, T> {
    const BLOCK_HEADER_LAYOUT: FieldLayout<BH, T> = FieldLayout::align_to(T::OBJECT_ALIGN);
    #[inline(always)]
    pub unsafe fn get_object(&mut self) -> &mut T {
        &mut *Self::BLOCK_HEADER_LAYOUT.to_adjacent(&mut self.0)
    }
    #[inline(always)]
    pub fn block_layout(size: usize) -> Layout {
        unsafe {
            Layout::from_size_align_unchecked(
                Self::BLOCK_HEADER_LAYOUT.size + size,
                Self::BLOCK_HEADER_LAYOUT.align,
            )
        }
    }
    #[inline(always)]
    pub unsafe fn delete(&mut self) {
        self.0.delete::<T>();
    }
}

pub trait BlockHeader: Sized {
    // required
    unsafe fn ref_update(&self, i: RefUpdate) -> isize;
    unsafe fn delete<T: Object>(&mut self);
    //
    #[inline(always)]
    unsafe fn block<T: Object>(&mut self) -> &mut Block<Self, T> {
        &mut *(self.as_mut_ptr() as *mut _)
    }
}

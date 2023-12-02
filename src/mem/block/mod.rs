pub mod header;

use core::{alloc::Layout, marker::PhantomData};

use self::header::BlockHeader;

use super::{field_layout::FieldLayout, object::Object};

#[repr(transparent)]
pub struct Block<BH: BlockHeader, T: Object> {
    pub header: BH,
    _0: PhantomData<T>,
}

impl<BH: BlockHeader, T: Object> Block<BH, T> {
    const BLOCK_HEADER_LAYOUT: FieldLayout<BH, T> = FieldLayout::align_to(T::OBJECT_ALIGN);
    #[inline(always)]
    pub unsafe fn object(&mut self) -> &mut T {
        &mut *Self::BLOCK_HEADER_LAYOUT.to_adjacent(&mut self.header)
    }
    #[inline(always)]
    pub fn block_layout(object_size: usize) -> Layout {
        unsafe {
            Layout::from_size_align_unchecked(
                Self::BLOCK_HEADER_LAYOUT.size + object_size,
                Self::BLOCK_HEADER_LAYOUT.align,
            )
        }
    }
    #[inline(always)]
    pub unsafe fn delete(&mut self) {
        BH::delete(self);
    }
}

pub mod header;

use core::{alloc::Layout, marker::PhantomData};

use self::header::BlockHeader;

use super::{
    field_layout::FieldLayout,
    manager::Manager,
    object::{holder::ObjectHolder, holder_mut::ObjectHolderMut, Object},
};

#[repr(transparent)]
pub struct Block<BH: BlockHeader, T: Object> {
    pub header: BH,
    _0: PhantomData<T>,
}

impl<BH: BlockHeader, T: Object> ObjectHolder for Block<BH, T> {
    type Object = T;
    #[inline(always)]
    fn object(&self) -> &Self::Object {
        unsafe { &*Self::BLOCK_HEADER_LAYOUT.to_adjacent(&self.header) }
    }
}

impl<BH: BlockHeader, T: Object> ObjectHolderMut for Block<BH, T> {
    #[inline(always)]
    fn mut_object(&mut self) -> &mut Self::Object {
        unsafe { &mut *Self::BLOCK_HEADER_LAYOUT.to_adjacent_mut(&mut self.header) }
    }
}

impl<BH: BlockHeader, T: Object> Block<BH, T> {
    const BLOCK_HEADER_LAYOUT: FieldLayout<BH, T> = FieldLayout::align_to(T::OBJECT_ALIGN);
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
        let object = self.mut_object();
        let object_size = object.object_size();
        object.object_drop_in_place();
        <BH::Manager as Manager>::dealloc(
            self as *mut _ as *mut u8,
            Self::block_layout(object_size),
        );
    }
}

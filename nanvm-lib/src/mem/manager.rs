use core::alloc::Layout;

use crate::common::ref_mut::RefMut;

use super::{
    block::{header::BlockHeader, Block},
    fixed::Fixed,
    flexible_array::{len::FlexibleArrayLen, new::FlexibleArrayNew, FlexibleArray},
    mut_ref::MutRef,
    new_in_place::NewInPlace,
};

/// Block = (Header, Object)
pub trait Manager: Sized {
    // required:
    type BlockHeader: BlockHeader;
    unsafe fn alloc(self, layout: Layout) -> *mut u8;
    unsafe fn dealloc(ptr: *mut u8, layout: Layout);
    // optional:
    /// Allocate a block of memory for a new T object and initialize the object with the `new_in_place`.
    fn new<N: NewInPlace>(self, new_in_place: N) -> MutRef<N::Result, Self> {
        unsafe {
            let p = self.alloc(Block::<Self::BlockHeader, N::Result>::block_layout(
                new_in_place.result_size(),
            )) as *mut Block<Self::BlockHeader, N::Result>;
            {
                let block = &mut *p;
                block
                    .header
                    .as_mut_ptr()
                    .write(Self::BlockHeader::default());
                new_in_place.new_in_place(block.object_mut());
            }
            MutRef::new(p)
        }
    }
    #[inline(always)]
    fn fixed_new<T>(self, value: T) -> MutRef<Fixed<T>, Self> {
        self.new(Fixed(value))
    }
    #[inline(always)]
    fn flexible_array_new<I>(
        self,
        items: impl ExactSizeIterator<Item = I>,
    ) -> MutRef<FlexibleArray<FlexibleArrayLen<I>>, Self> {
        self.new(FlexibleArrayNew::from(items))
    }
}

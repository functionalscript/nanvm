use core::alloc::Layout;

use crate::common::ref_mut::RefMut;

use super::{
    block::{header::BlockHeader, Block},
    fixed::Fixed,
    flexible_array::{len::FlexibleArrayLen, new::FlexibleArrayNew, FlexibleArray},
    mut_ref::MutRef,
    new_in_place::NewInPlace,
};

pub trait Dealloc {
    type BlockHeader: BlockHeader;
    unsafe fn dealloc(ptr: *mut u8, layout: Layout);
}

/// Block = (Header, Object)
pub trait Manager: Sized {
    // required:
    type Dealloc: Dealloc;
    unsafe fn alloc(self, layout: Layout) -> *mut u8;
    // optional:
    /// Allocate a block of memory for a new T object and initialize the object with the `new_in_place`.
    fn new<N: NewInPlace>(self, new_in_place: N) -> MutRef<N::Result, Self::Dealloc> {
        unsafe {
            let p = self.alloc(Block::<N::Result, Self::Dealloc>::block_layout(
                new_in_place.result_size(),
            )) as *mut Block<N::Result, Self::Dealloc>;
            {
                let block = &mut *p;
                block
                    .header
                    .as_mut_ptr()
                    .write(<<Self as Manager>::Dealloc as Dealloc>::BlockHeader::default());
                new_in_place.new_in_place(block.object_mut());
            }
            MutRef::new(p)
        }
    }
    #[inline(always)]
    fn fixed_new<T>(self, value: T) -> MutRef<Fixed<T>, Self::Dealloc> {
        self.new(Fixed(value))
    }
    #[inline(always)]
    fn flexible_array_new<I>(
        self,
        items: impl ExactSizeIterator<Item = I>,
    ) -> MutRef<FlexibleArray<FlexibleArrayLen<I>>, Self::Dealloc> {
        self.new(FlexibleArrayNew::from(items))
    }
}

use core::alloc::Layout;
use core::fmt::Debug;

use crate::common::ref_mut::RefMut;

use super::{
    block::Block,
    block_header::BlockHeader,
    constructor::Constructor,
    fixed::Fixed,
    flexible_array::{constructor::FlexibleArrayConstructor, FlexibleArray},
    mut_ref::MutRef,
};

pub trait Dealloc: Debug {
    type BlockHeader: BlockHeader;
    unsafe fn dealloc(ptr: *mut u8, layout: Layout);
}

/// Block = (Header, Object)
pub trait Manager: Sized + Copy + Debug {
    // required:
    type Dealloc: Dealloc;

    unsafe fn alloc(self, layout: Layout) -> *mut u8;
    // optional:
    /// Allocate a block of memory for a new T object and initialize the object with the `new_in_place`.
    #[allow(clippy::wrong_self_convention)]
    fn new<N: Constructor>(self, new_in_place: N) -> MutRef<N::Result, Self::Dealloc> {
        unsafe {
            let p = self.alloc(Block::<N::Result, Self::Dealloc>::block_layout(
                new_in_place.result_size(),
            )) as *mut Block<N::Result, Self::Dealloc>;
            {
                let block = &mut *p;
                block
                    .header
                    .to_mut_ptr()
                    .write(<<Self as Manager>::Dealloc as Dealloc>::BlockHeader::default());
                new_in_place.construct(block.object_mut());
            }
            MutRef::new(p)
        }
    }
    #[inline(always)]
    fn fixed_new<T>(self, value: T) -> MutRef<Fixed<T>, Self::Dealloc> {
        self.new(Fixed(value))
    }
    #[inline(always)]
    fn flexible_array_new<I, E: ExactSizeIterator<Item = I>>(
        self,
        items: impl IntoIterator<IntoIter = E>,
    ) -> MutRef<FlexibleArray<I, usize>, Self::Dealloc> {
        self.new(FlexibleArrayConstructor::from(items.into_iter()))
    }
}

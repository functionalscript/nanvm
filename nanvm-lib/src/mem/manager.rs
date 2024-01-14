use core::{alloc::Layout, cmp::Ordering, ptr::copy_nonoverlapping};

use crate::{common::ref_mut::RefMut, mem::object::Object};

use super::{
    block::Block,
    block_header::BlockHeader,
    constructor::{Assign, Constructor},
    fixed::Fixed,
    flexible_array::{constructor::FlexibleArrayConstructor, FlexibleArray},
    mut_ref::MutRef,
};

pub trait Dealloc {
    type BlockHeader: BlockHeader;
    unsafe fn dealloc(ptr: *mut u8, layout: Layout);
}

/// Block = (Header, Object)
pub trait Manager: Sized + Copy {
    // required:
    type Dealloc: Dealloc;
    unsafe fn alloc(self, layout: Layout) -> *mut u8;
    // optional methods:
    #[inline(always)]
    unsafe fn typed_alloc<O: Object>(self, object_size: usize) -> *mut Block<O, Self::Dealloc> {
        self.alloc(Block::<O, Self::Dealloc>::block_layout(object_size)) as _
    }
    /// A user must call destructors for all old exceeding objects in the block before calling this method and
    /// initialize all new extra objects after calling this method.
    #[inline(always)]
    unsafe fn realloc(self, ptr: *mut u8, old_layout: Layout, new_layout: Layout) -> *mut u8 {
        self.realloc_move(ptr, old_layout, new_layout)
    }
    /// A default implementation of the `realloc` method.
    unsafe fn realloc_move(self, ptr: *mut u8, old_layout: Layout, new_layout: Layout) -> *mut u8 {
        let new_ptr = self.alloc(new_layout);
        let size = old_layout.size().min(new_layout.size());
        copy_nonoverlapping(ptr, new_ptr, size);
        Self::Dealloc::dealloc(ptr, old_layout);
        new_ptr
    }
    #[inline(always)]
    unsafe fn typed_realloc<O: Object>(
        self,
        ptr: *mut Block<O, Self::Dealloc>,
        old_size: usize,
        new_size: usize,
    ) -> *mut Block<O, Self::Dealloc> {
        let block_layout = |size| Block::<O, Self::Dealloc>::block_layout(size);
        self.realloc(ptr as _, block_layout(old_size), block_layout(new_size)) as _
    }
    /// Allocate a block of memory for a new T object and initialize the object with the `new_in_place`.
    fn new<C: Constructor>(self, constructor: C) -> MutRef<C::Object, Self::Dealloc> {
        unsafe {
            let p = self.typed_alloc(constructor.new_size());
            {
                let block = &mut *p;
                block
                    .header
                    .to_mut_ptr()
                    .write(<<Self as Manager>::Dealloc as Dealloc>::BlockHeader::default());
                constructor.construct(block.object_mut());
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
    ) -> MutRef<FlexibleArray<I, usize>, Self::Dealloc> {
        self.new(FlexibleArrayConstructor::from(items))
    }
    fn resize<A: Assign>(self, m: &mut MutRef<A::Object, Self::Dealloc>, a: A) {
        let old_size = m.object_size();
        let new_size = a.new_size();
        let realloc = |m: &mut MutRef<A::Object, _>| unsafe {
            let p = self.typed_realloc(m.internal(), old_size, new_size);
            m.set_internal(p);
        };
        let assign = |m: &mut MutRef<A::Object, _>| unsafe { a.assign(m.internal().object_mut()) };
        match old_size.cmp(&new_size) {
            Ordering::Equal => assign(m),
            Ordering::Greater => {
                assign(m);
                realloc(m);
            }
            Ordering::Less => {
                realloc(m);
                assign(m);
            }
        }
    }
}

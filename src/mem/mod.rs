mod block_header;
mod field_layout;
mod fixed;
mod flexible_array;
mod new_in_place;
mod object;
mod rc_update;
mod ref_;

use core::{
    alloc::Layout,
    marker::PhantomData,
    ptr::drop_in_place,
    sync::atomic::{AtomicIsize, Ordering},
};
use std::alloc::{alloc, dealloc};

use self::{
    block_header::BlockHeader, field_layout::FieldLayout, new_in_place::NewInPlace, object::Object,
    rc_update::RcUpdate, ref_::Ref,
};

/// Block = (Header, Object)
trait Manager: Sized {
    type BlockHeader: BlockHeader;
    /// Allocate a block of memory for a new T object and initialize the object with the `new_in_place`.
    unsafe fn new<N: NewInPlace>(self, new_in_place: N) -> Ref<N::Result, Self>;
}

struct Global();

/// Every object type has its own header layout which depends on the type's alignment.
trait GlobalLayout: Object {
    const HEADER_LAYOUT: FieldLayout<GlobalHeader, Self> =
        FieldLayout::align_to(Self::OBJECT_ALIGN);
}

impl<T: Object> GlobalLayout for T {}

struct GlobalHeader(AtomicIsize);

impl GlobalHeader {
    #[inline(always)]
    const unsafe fn block_layout<T: Object>(size: usize) -> Layout {
        Layout::from_size_align_unchecked(T::HEADER_LAYOUT.size + size, T::HEADER_LAYOUT.align)
    }
}

impl BlockHeader for GlobalHeader {
    #[inline(always)]
    unsafe fn rc_update(&self, i: RcUpdate) -> isize {
        self.0.fetch_add(i as isize, Ordering::Relaxed)
    }
    #[inline(always)]
    unsafe fn get_object<T: Object>(&mut self) -> &mut T {
        &mut *T::HEADER_LAYOUT.to_adjacent(self)
    }
    unsafe fn delete<T: Object>(&mut self) {
        let p = self.get_object::<T>();
        let size = p.object_size();
        drop_in_place(p);
        dealloc(p as *mut T as *mut u8, Self::block_layout::<T>(size));
    }
}

impl Manager for Global {
    type BlockHeader = GlobalHeader;
    unsafe fn new<N: NewInPlace>(self, new_in_place: N) -> Ref<N::Result, Self> {
        let header_p = alloc(Self::BlockHeader::block_layout::<N::Result>(
            new_in_place.result_size(),
        )) as *mut Self::BlockHeader;
        *header_p = GlobalHeader(AtomicIsize::new(1));
        new_in_place.new_in_place((*header_p).get_object());
        Ref::new(header_p)
    }
}

#[cfg(test)]
mod test {
    use std::mem::{align_of, size_of};

    struct MyStruct {
        a: u8,  // 1 byte
        b: u16, // 2 bytes
        c: u8,  // 1 byte
        d: u8,
    }

    const _: () = assert!(size_of::<MyStruct>() == 6);
    const _: () = assert!(align_of::<MyStruct>() == 2);
}

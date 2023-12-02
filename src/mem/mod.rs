mod field_layout;
mod fixed;
mod new_in_place;
mod object;
mod rc_update;
mod variable;

use core::{
    alloc::Layout,
    marker::PhantomData,
    ptr::drop_in_place,
    sync::atomic::{AtomicIsize, Ordering},
};
use std::alloc::{alloc, dealloc};

use self::{
    field_layout::FieldLayout, new_in_place::NewInPlace, object::Object, rc_update::RcUpdate,
};

/// Block header
trait BlockHeader {
    unsafe fn rc_update(&self, i: RcUpdate) -> isize;
    unsafe fn get<T: Object>(&mut self) -> &mut T;
    unsafe fn delete<T: Object>(&mut self);
}

/// Block = (Header, Object)
trait Manager: Sized {
    type BlockHeader: BlockHeader;
    /// Allocate a block of memory for a new T object and initialize the object with the `new_in_place`.
    unsafe fn new<N: NewInPlace>(self, new_in_place: N) -> Ref<N::Object, Self>;
}

/// A reference to an object allocated by a memory manager.
#[repr(transparent)]
struct Ref<T: Object, M: Manager>(*mut M::BlockHeader, PhantomData<T>);

impl<T: Object, M: Manager> Clone for Ref<T, M> {
    fn clone(&self) -> Self {
        let v = self.0;
        unsafe { (*v).rc_update(RcUpdate::AddRef) };
        Self(v, PhantomData)
    }
}

impl<T: Object, M: Manager> Drop for Ref<T, M> {
    fn drop(&mut self) {
        unsafe {
            let p = &mut *self.0;
            if p.rc_update(RcUpdate::Release) == 0 {
                p.delete::<T>();
            }
        }
    }
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
    unsafe fn get<T: Object>(&mut self) -> &mut T {
        &mut *T::HEADER_LAYOUT.to_adjacent(self)
    }
    unsafe fn delete<T: Object>(&mut self) {
        let p = self.get::<T>();
        let size = p.object_size();
        drop_in_place(p);
        dealloc(p as *mut T as *mut u8, Self::block_layout::<T>(size));
    }
}

impl Manager for Global {
    type BlockHeader = GlobalHeader;
    unsafe fn new<N: NewInPlace>(self, new_in_place: N) -> Ref<N::Object, Self> {
        let header_p = alloc(Self::BlockHeader::block_layout::<N::Object>(
            new_in_place.size(),
        )) as *mut Self::BlockHeader;
        *header_p = GlobalHeader(AtomicIsize::new(1));
        new_in_place.new_in_place(header_p as *mut N::Object);
        Ref(header_p, PhantomData)
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

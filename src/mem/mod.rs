mod object;

use core::{
    alloc::Layout,
    marker::PhantomData,
    mem::{align_of, size_of},
    ptr::drop_in_place,
    sync::atomic::{AtomicIsize, Ordering},
};
use std::alloc::{alloc, dealloc};

use crate::common::usize::max;

use self::object::Object;

/// Update for a reference counter
enum RcUpdate {
    AddRef = 1,
    Release = -1,
}

/// Block header
trait Header {
    unsafe fn rc_update(&self, i: RcUpdate) -> isize;
    unsafe fn get<T: Object>(&mut self) -> &mut T;
    unsafe fn delete<T: Object>(&mut self);
}

/// Block = (Header, Object)
trait Manager: Sized {
    type Header: Header;
    /// Allocate a block of memory for a new T object and initialize the object with `init`.
    unsafe fn new<T: Object, F: FnOnce(*mut T)>(self, size: usize, init: F) -> Ref<T, Self>;
}

/// A reference to an object allocated by a memory manager.
#[repr(transparent)]
struct Ref<T: Object, M: Manager>(*mut M::Header, PhantomData<T>);

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

pub const fn aligned_size<T>(align: usize) -> usize {
    let mask = align - 1;
    (size_of::<T>() + mask) & !mask
}

pub const fn aligned_layout<T>(align: usize) -> Layout {
    unsafe {
        Layout::from_size_align_unchecked(aligned_size::<T>(align), max(align_of::<T>(), align))
    }
}

struct Global();

/// Every object type has its own header layout which depends on the type's alignment.
trait GlobalLayout: Object {
    const HEADER_LAYOUT: Layout = aligned_layout::<GlobalHeader>(Self::ALIGN);
    const HEADER_LAYOUT_ALIGN: usize = Self::HEADER_LAYOUT.align();
    const HEADER_LAYOUT_SIZE: usize = Self::HEADER_LAYOUT.size();
}

impl<T: Object> GlobalLayout for T {}

struct GlobalHeader(AtomicIsize);

impl GlobalHeader {
    #[inline(always)]
    const unsafe fn block_layout<T: Object>(size: usize) -> Layout {
        Layout::from_size_align_unchecked(T::HEADER_LAYOUT_SIZE + size, T::HEADER_LAYOUT_ALIGN)
    }
}

impl Header for GlobalHeader {
    #[inline(always)]
    unsafe fn rc_update(&self, i: RcUpdate) -> isize {
        self.0.fetch_add(i as isize, Ordering::Relaxed)
    }
    #[inline(always)]
    unsafe fn get<T: Object>(&mut self) -> &mut T {
        let p = self as *mut Self as *mut u8;
        let p = p.add(T::HEADER_LAYOUT_SIZE) as *mut T;
        &mut *p
    }
    unsafe fn delete<T: Object>(&mut self) {
        let p = self.get::<T>();
        let size = p.size();
        drop_in_place(p);
        dealloc(p as *mut T as *mut u8, Self::block_layout::<T>(size));
    }
}

impl Manager for Global {
    type Header = GlobalHeader;
    unsafe fn new<T: Object, F: FnOnce(*mut T)>(self, size: usize, init: F) -> Ref<T, Self> {
        let header_p = alloc(Self::Header::block_layout::<T>(size)) as *mut Self::Header;
        *header_p = GlobalHeader(AtomicIsize::new(1));
        init((*header_p).get::<T>());
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

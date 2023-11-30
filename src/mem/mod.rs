use core::{
    alloc::Layout,
    marker::PhantomData,
    mem::{align_of, size_of},
    ptr::drop_in_place,
    sync::atomic::{AtomicIsize, Ordering},
};
use std::alloc::{alloc, dealloc};

use crate::common::usize::max;

// Memory Layout

trait MemLayout {
    const ALIGN: usize;
    fn size(&self) -> usize;
    unsafe fn drop_in_place(&mut self);
}

impl<T> MemLayout for T {
    const ALIGN: usize = align_of::<T>();
    fn size(&self) -> usize {
        size_of::<T>()
    }
    unsafe fn drop_in_place(&mut self) {
        drop_in_place(self)
    }
}

enum Update {
    AddRef = 1,
    Release = -1,
}

trait Header {
    unsafe fn update(&self, i: Update) -> isize;
    unsafe fn get<T>(&mut self) -> &mut T;
    unsafe fn delete<T>(&mut self);
}

trait Manager: Sized {
    type Header: Header;
    unsafe fn new<T: MemLayout, F: FnOnce(*mut T)>(self, size: usize, init: F) -> Ref<T, Self>;
}

#[repr(transparent)]
struct Ref<T, M: Manager>(*mut M::Header, PhantomData<T>);

impl<T, M: Manager> Clone for Ref<T, M> {
    fn clone(&self) -> Self {
        let v = self.0;
        unsafe { (*v).update(Update::AddRef) };
        Self(v, PhantomData)
    }
}

impl<T, M: Manager> Drop for Ref<T, M> {
    fn drop(&mut self) {
        unsafe {
            let p = &mut *self.0;
            if p.update(Update::Release) == 0 {
                p.delete::<T>();
            }
        }
    }
}

const fn aligned_size<T>(align: usize) -> usize {
    let mask = align - 1;
    (size_of::<T>() + mask) & !mask
}

const fn aligned_layout<T>(align: usize) -> Layout {
    unsafe {
        Layout::from_size_align_unchecked(aligned_size::<T>(align), max(align_of::<T>(), align))
    }
}

struct Global();

/// Every type has its own header layout which depends on the type's alignment.
trait GlobalLayout: MemLayout {
    const HEADER_LAYOUT: Layout = aligned_layout::<GlobalHeader>(Self::ALIGN);
    const HEADER_LAYOUT_ALIGN: usize = Self::HEADER_LAYOUT.align();
    const HEADER_LAYOUT_SIZE: usize = Self::HEADER_LAYOUT.size();
}

impl<T: MemLayout> GlobalLayout for T {}

struct GlobalHeader(AtomicIsize);

impl GlobalHeader {
    #[inline(always)]
    const unsafe fn wrap_layout<T>(size: usize) -> Layout {
        Layout::from_size_align_unchecked(T::HEADER_LAYOUT_SIZE + size, T::HEADER_LAYOUT_ALIGN)
    }
}

impl Header for GlobalHeader {
    #[inline(always)]
    unsafe fn update(&self, i: Update) -> isize {
        self.0.fetch_add(i as isize, Ordering::Relaxed)
    }
    #[inline(always)]
    unsafe fn get<T>(&mut self) -> &mut T {
        &mut *(self as *mut Self as *mut T).add(T::HEADER_LAYOUT.size())
    }
    unsafe fn delete<T>(&mut self) {
        let p = self.get::<T>();
        let size = p.size();
        drop_in_place(p);
        dealloc(p as *mut T as *mut u8, Self::wrap_layout::<T>(size));
    }
}

impl Manager for Global {
    type Header = GlobalHeader;
    unsafe fn new<T: MemLayout, F: FnOnce(*mut T)>(self, size: usize, init: F) -> Ref<T, Self> {
        let header_p = alloc(Self::Header::wrap_layout::<T>(size)) as *mut Self::Header;
        *header_p = GlobalHeader(AtomicIsize::new(1));
        init((*header_p).get::<T>());
        Ref(header_p, PhantomData)
    }
}

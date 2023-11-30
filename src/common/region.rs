use core::{
    alloc::Layout,
    marker::PhantomData,
    mem::{align_of, size_of},
    ptr::drop_in_place,
    sync::atomic::{AtomicIsize, Ordering},
};
use std::alloc::{alloc, dealloc};

use super::usize::max;

// Region Layout

trait RegionLayout {
    const ALIGN: usize;
    fn size(&self) -> usize;
    unsafe fn drop_in_place(&mut self);
}

impl<T> RegionLayout for T {
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

trait Region: Sized {
    type Header: Header;
    unsafe fn new<T: RegionLayout, F: FnOnce(*mut T)>(self, size: usize, init: F) -> Ref<T, Self>;
}

#[repr(transparent)]
struct Ref<T, R: Region>(*mut R::Header, PhantomData<T>);

impl<T, R: Region> Clone for Ref<T, R> {
    fn clone(&self) -> Self {
        let v = self.0;
        unsafe { (*v).update(Update::AddRef) };
        Self(v, PhantomData)
    }
}

impl<T, R: Region> Drop for Ref<T, R> {
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

struct GlobalRegion();

/// Every type has its own header layout which depends on the type's alignment.
trait GlobalRegionLayout: RegionLayout {
    const HEADER_LAYOUT: Layout = aligned_layout::<GlobalRegionHeader>(Self::ALIGN);
    const HEADER_LAYOUT_ALIGN: usize = Self::HEADER_LAYOUT.align();
    const HEADER_LAYOUT_SIZE: usize = Self::HEADER_LAYOUT.size();
}

impl<T: RegionLayout> GlobalRegionLayout for T {}

struct GlobalRegionHeader(AtomicIsize);

impl GlobalRegionHeader {
    const ORDER: Ordering = Ordering::Relaxed;
    #[inline(always)]
    const unsafe fn wrap_layout<T>(size: usize) -> Layout {
        Layout::from_size_align_unchecked(T::HEADER_LAYOUT_SIZE + size, T::HEADER_LAYOUT_ALIGN)
    }
}

impl Header for GlobalRegionHeader {
    #[inline(always)]
    unsafe fn update(&self, i: Update) -> isize {
        self.0.fetch_add(i as isize, Self::ORDER)
    }
    #[inline(always)]
    unsafe fn get<T>(&mut self) -> &mut T {
        &mut *(self as *mut Self as *mut T).add(T::HEADER_LAYOUT.size())
    }
    unsafe fn delete<T>(&mut self) {
        let p = self.get::<T>();
        drop_in_place(p);
        dealloc(p as *mut T as *mut u8, Self::wrap_layout::<T>(p.size()));
    }
}

impl Region for GlobalRegion {
    type Header = GlobalRegionHeader;
    unsafe fn new<T: RegionLayout, F: FnOnce(*mut T)>(self, size: usize, init: F) -> Ref<T, Self> {
        let header_p = alloc(Self::Header::wrap_layout::<T>(size)) as *mut Self::Header;
        *header_p = GlobalRegionHeader(AtomicIsize::new(1));
        init((*header_p).get::<T>());
        Ref(header_p, PhantomData)
    }
}

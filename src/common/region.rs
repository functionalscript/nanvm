use core::{
    marker::PhantomData,
    mem::{align_of, size_of},
    ptr::drop_in_place,
};
use std::{
    alloc::{alloc, dealloc, Layout},
    sync::atomic::{AtomicIsize, Ordering},
};

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

struct GlobalRegionHeader(AtomicIsize);

/*
trait GlobalRegionLayout: RegionLayout {
    const RC_LAYOUT: Layout = aligned_layout::<Self::Rc>(Self::ALIGN);
}

impl<T: RegionLayout> GlobalRegionLayout for T {}

impl GlobalRegion {
    const ORDER: Ordering = Ordering::Relaxed;
    #[inline(always)]
    const unsafe fn rc_ptr<T: RegionLayout>(p: *mut T) -> *mut Rc {
        p.sub(T::RC_LAYOUT.size()) as *mut Rc
    }
    #[inline(always)]
    unsafe fn rc_update(p: &Rc, i: isize) -> isize {
        p.fetch_add(i, Self::ORDER)
    }
    #[inline(always)]
    const unsafe fn layout<T>(size: usize) -> Layout {
        Layout::from_size_align_unchecked(T::RC_LAYOUT.size() + size, T::RC_LAYOUT.align())
    }
}

impl Region for GlobalRegion {
    type Header = AtomicIsize;
    unsafe fn alloc<T: RegionLayout, F: FnOnce(*mut T)>(
        self,
        size: usize,
        init: F,
    ) -> Ref<T, Self> {
        let ref_counter_p = alloc(Self::layout::<T>(size)) as *mut Rc;
        (*ref_counter_p).store(1, Self::ORDER);
        let p = ref_counter_p.add(T::RC_LAYOUT.size()) as *mut T;
        init(p);
        Ref(p, PhantomData)
    }

    unsafe fn add_ref<T: RegionLayout>(p: *mut T) {
        Self::rc_update(&*Self::rc_ptr(p), 1);
    }

    unsafe fn release<T: RegionLayout>(p: *mut T) {
        let rcp = Self::rc_ptr(p);
        if Self::rc_update(&*rcp, -1) == 0 {
            p.drop_in_place();
            dealloc(rcp as *mut u8, Self::layout::<T>((*p).size()));
        }
    }
}
*/

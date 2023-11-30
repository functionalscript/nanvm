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

trait Region: Sized {
    unsafe fn alloc<T: RegionLayout, F: FnOnce(*mut T)>(self, size: usize, init: F)
        -> Ref<T, Self>;
    unsafe fn add_ref<T: RegionLayout>(p: *mut T);
    unsafe fn release<T: RegionLayout>(p: *mut T);
}

struct Ref<T, R: Region>(*mut T, PhantomData<R>);

impl<T, R: Region> Clone for Ref<T, R> {
    fn clone(&self) -> Self {
        let v = self.0;
        unsafe { R::add_ref(v) };
        Self(v, PhantomData)
    }
}

impl<T, R: Region> Drop for Ref<T, R> {
    fn drop(&mut self) {
        unsafe { R::release(self.0) };
    }
}

const fn aligned_layout<T>(align: usize) -> Layout {
    let align = max(size_of::<T>(), align);
    let mask = max(size_of::<T>(), align) - 1;
    unsafe { Layout::from_size_align_unchecked((size_of::<T>() + mask) & !mask, align) }
}

struct GlobalRegion();

type Rc = AtomicIsize;

trait GlobalRegionLayout: RegionLayout {
    const RC_LAYOUT: Layout = aligned_layout::<Rc>(Self::ALIGN);
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

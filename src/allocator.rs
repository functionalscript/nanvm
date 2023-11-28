use std::alloc::{alloc, dealloc, Layout};

pub trait Allocator: Clone {
    unsafe fn alloc(self, layout: Layout) -> *mut u8;
    unsafe fn dealloc(self, ptr: *mut u8, layout: Layout);
}

#[derive(Clone)]
pub struct GlobalAllocator();

impl Allocator for GlobalAllocator {
    unsafe fn alloc(self, layout: Layout) -> *mut u8 {
        alloc(layout)
    }
    unsafe fn dealloc(self, ptr: *mut u8, layout: Layout) {
        dealloc(ptr, layout)
    }
}

use std::alloc::{alloc, dealloc, Layout};

pub trait Allocator {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8;
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout);
}

#[derive(Clone, Copy)]
pub struct GlobalAllocator();

impl Allocator for GlobalAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        alloc(layout)
    }
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        dealloc(ptr, layout)
    }
}

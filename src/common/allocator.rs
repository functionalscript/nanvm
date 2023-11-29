use std::alloc::{alloc, dealloc, Layout};

pub trait Allocator {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8;
    /// Note: the function shouldn't accept `&mut self` because
    /// the `ptr` should provide all required information, if needed.
    unsafe fn dealloc(ptr: *mut u8, layout: Layout);
}

#[derive(Clone, Copy)]
pub struct GlobalAllocator();

impl Allocator for GlobalAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        alloc(layout)
    }
    unsafe fn dealloc(ptr: *mut u8, layout: Layout) {
        dealloc(ptr, layout)
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[test]
    #[wasm_bindgen_test]
    fn test_allocator() {
        let mut allocator = GlobalAllocator();
        let layout = Layout::from_size_align(1, 1).unwrap();
        let ptr = unsafe { allocator.alloc(layout) };
        assert!(!ptr.is_null());
        unsafe { GlobalAllocator::dealloc(ptr, layout) };
    }

    struct X<T>(T);

    #[test]
    fn test_internal() {
        let m = {
            let a = 5;
            let x = X(&a);
            // x
        };
    }
}

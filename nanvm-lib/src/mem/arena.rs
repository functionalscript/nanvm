use core::{alloc::Layout, cell::Cell, ops::Range};

use super::{
    block::header::BlockHeader,
    field_layout::align_to,
    manager::{Dealloc, Manager},
    ref_::counter_update::RefCounterUpdate,
};

trait Buffer {
    fn items_mut(&mut self) -> &mut [u8];
    unsafe fn range(&mut self) -> Range<*mut u8> {
        self.items_mut().as_mut_ptr_range()
    }
}

impl Buffer for &mut [u8] {
    #[inline(always)]
    fn items_mut(&mut self) -> &mut [u8] {
        self
    }
}

#[derive(Debug)]
struct Arena<T: Buffer> {
    buffer: T,
    current: Cell<usize>,
    end: usize,
}

impl<T: Buffer> Arena<T> {
    #[inline(always)]
    const fn layout(size: usize) -> Layout {
        unsafe { Layout::from_size_align_unchecked(size, 1) }
    }
    pub fn new(mut buffer: T) -> Self {
        let range = unsafe { buffer.range() };
        let current = Cell::new(range.start as usize);
        Self {
            buffer,
            current,
            end: range.end as usize,
        }
    }
}

#[derive(Default)]
struct NoHeader();

impl BlockHeader for NoHeader {
    #[inline(always)]
    unsafe fn ref_counter_update(&self, _val: RefCounterUpdate) -> isize {
        1
    }
}

impl<T: Buffer> Dealloc for &Arena<T> {
    type BlockHeader = NoHeader;
    #[inline(always)]
    unsafe fn dealloc(_: *mut u8, _: Layout) {}
}

impl<T: Buffer> Manager for &Arena<T> {
    type Dealloc = Self;
    unsafe fn alloc(self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let current = align_to(self.current.get() as usize, align);
        let end = current + layout.size();
        if end > self.end as usize {
            panic!("out of memory");
        }
        self.current.set(end);
        current as *mut u8
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{arena::Buffer, manager::Manager};

    use super::Arena;

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        let mut range = [0u8; 1024];
        let arena = Arena::new(&mut range[..]);
        assert_eq!(arena.end as usize - arena.current.get(), 1024);
        let r = arena.fixed_new(42u8).to_ref();
        let r2 = r.try_to_mut_ref().unwrap_err();
        assert_eq!(arena.end as usize - arena.current.get(), 1023);
        let mr = arena.fixed_new(43);
        assert_eq!(arena.end as usize - arena.current.get(), 1016);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_1() {
        let mut range = [0u8; 1];
        let arena = Arena::new(&mut range[..]);
        assert_eq!(arena.end as usize - arena.current.get(), 1);
        let r = arena.fixed_new(42u8).to_ref();
        assert_eq!(arena.end as usize, arena.current.get());
    }

    #[test]
    #[should_panic]
    #[wasm_bindgen_test]
    fn test_out_of_memory() {
        let mut range = [0u8; 1];
        let arena = Arena::new(&mut range[..]);
        let r = arena.fixed_new(42u8).to_ref();
        let r2 = arena.fixed_new(42u8).to_ref();
    }
}

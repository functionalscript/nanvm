use core::{alloc::Layout, cell::Cell, marker::PhantomData};

use super::{
    block::header::BlockHeader,
    field_layout::align_to,
    manager::{Dealloc, Manager},
    ref_::counter_update::RefCounterUpdate,
};

#[derive(Debug)]
struct Arena<'a> {
    begin: Cell<usize>,
    end: usize,
    _0: PhantomData<&'a ()>,
}

impl<'a> Arena<'a> {
    #[inline(always)]
    const fn layout(size: usize) -> Layout {
        unsafe { Layout::from_size_align_unchecked(size, 1) }
    }
    pub fn new(range: &'a mut [u8]) -> Self {
        let begin = range.as_ptr() as usize;
        Self {
            begin: Cell::new(begin),
            end: begin + range.len(),
            _0: PhantomData,
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

impl<'a> Dealloc for &Arena<'a> {
    type BlockHeader = NoHeader;
    #[inline(always)]
    unsafe fn dealloc(_: *mut u8, _: Layout) {}
}

impl<'a> Manager for &'a Arena<'a> {
    type Dealloc = Self;
    unsafe fn alloc(self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let current = align_to(self.begin.get(), align);
        let end = current + layout.size();
        if end > self.end {
            panic!("out of memory");
        }
        self.begin.set(end);
        current as *mut u8
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::manager::Manager;

    use super::Arena;

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        let mut range = [0u8; 1024];
        let arena = Arena::new(&mut range);
        {
            assert_eq!(arena.end - arena.begin.get(), 1024);
            let r = arena.fixed_new(42u8).to_ref();
            let r2 = r.try_to_mut_ref().unwrap_err();
            assert_eq!(arena.end - arena.begin.get(), 1023);
            let mr = arena.fixed_new(43);
            assert_eq!(arena.end - arena.begin.get(), 1016);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_1() {
        let mut range = [0u8; 1];
        let arena = Arena::new(&mut range);
        assert_eq!(arena.end - arena.begin.get(), 1);
        let r = arena.fixed_new(42u8).to_ref();
        assert_eq!(arena.end, arena.begin.get());
    }

    #[test]
    #[should_panic]
    #[wasm_bindgen_test]
    fn test_out_of_memory() {
        let mut range = [0u8; 1];
        let arena = Arena::new(&mut range);
        let r = arena.fixed_new(42u8).to_ref();
        let r2 = arena.fixed_new(42u8).to_ref();
    }
}

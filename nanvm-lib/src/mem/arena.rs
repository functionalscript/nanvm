use core::{alloc::Layout, cell::Cell, marker::PhantomData};

use super::{
    block_header::BlockHeader,
    manager::{Dealloc, Manager},
    ref_counter_update::RefCounterUpdate,
};

#[derive(Debug)]
struct Arena<'a> {
    start: Cell<*mut u8>,
    end: *mut u8,
    _0: PhantomData<&'a mut [u8]>,
}

impl<'a> Arena<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        let range = buffer.as_mut_ptr_range();
        Self {
            start: Cell::new(range.start),
            end: range.end,
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

impl<'a> Dealloc for Arena<'a> {
    type BlockHeader = NoHeader;
    #[inline(always)]
    unsafe fn dealloc(_: *mut u8, _: Layout) {}
}

impl<'a> Manager for Arena<'a> {
    type Dealloc = Self;
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let result = {
            let result = self.start.get();
            result.byte_add(result.align_offset(layout.align()))
        };
        {
            let start = result.byte_add(layout.size());
            if start > self.end {
                panic!("out of memory");
            }
            self.start.set(start);
        }
        result
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
        //let x = {
            let mut range = [0u8; 1024];
            let arena = Arena::new(&mut range[..]);
            assert_eq!(arena.end as usize - arena.start.get() as usize, 1024);
            let r = arena.fixed_new(42u8).to_ref();
            let _r2 = r.try_to_mut_ref().unwrap_err();
            assert_eq!(arena.end as usize - arena.start.get() as usize, 1023);
            let _mr = arena.fixed_new(43);
            assert_eq!(arena.end as usize - arena.start.get() as usize, 1016);
            //_mr
        //};
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_1() {
        let mut range = [0u8; 1];
        let arena = Arena::new(&mut range[..]);
        assert_eq!(arena.end as usize - arena.start.get() as usize, 1);
        let _r = arena.fixed_new(42u8).to_ref();
        assert_eq!(arena.end as usize, arena.start.get() as usize);
    }

    #[test]
    #[should_panic]
    #[wasm_bindgen_test]
    fn test_out_of_memory() {
        let mut range = [0u8; 1];
        let arena = Arena::new(&mut range[..]);
        let _r = arena.fixed_new(42u8).to_ref();
        let _r2 = arena.fixed_new(42u8).to_ref();
    }
}

use core::{alloc::Layout, cell::RefCell, marker::PhantomData};

use super::{
    block::header::BlockHeader, field_layout::align_to, manager::Manager, ref_::update::RefUpdate,
};

#[derive(Debug)]
struct Arena<M: Manager> {
    begin: usize,
    end: usize,
    current: RefCell<usize>,
    _0: PhantomData<M>,
}

impl<M: Manager> Arena<M> {
    #[inline(always)]
    const fn layout(size: usize) -> Layout {
        unsafe { Layout::from_size_align_unchecked(size, 1) }
    }
    pub fn new(manager: M, size: usize) -> Arena<M> {
        let begin = unsafe { manager.alloc(Self::layout(size)) } as usize;
        Arena {
            begin,
            end: begin + size,
            current: RefCell::new(begin),
            _0: PhantomData,
        }
    }
}

impl<M: Manager> Drop for Arena<M> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { M::dealloc(self.begin as *mut u8, Self::layout(self.end - self.begin)) };
    }
}

#[derive(Default)]
struct NoHeader();

impl BlockHeader for NoHeader {
    #[inline(always)]
    unsafe fn ref_update(&mut self, _val: RefUpdate) -> isize {
        1
    }
}

impl<M: Manager> Manager for &Arena<M> {
    type BlockHeader = NoHeader;
    unsafe fn alloc(self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let mut c = self.current.borrow_mut();
        let current = align_to(*c, align);
        let end = current + layout.size();
        if end > self.end {
            panic!("out of memory");
        }
        *c = end;
        current as *mut u8
    }
    #[inline(always)]
    unsafe fn dealloc(_: *mut u8, _: Layout) {}
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{global::GLOBAL, manager::Manager};

    use super::Arena;

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        let arena = Arena::new(GLOBAL, 1024);
        {
            assert_eq!(arena.begin, *arena.current.borrow());
            assert_eq!(arena.end - arena.begin, 1024);
            let r = arena.fixed_new(42u8).to_ref();
            let r2 = r.try_to_mut_ref().unwrap_err();
            assert_eq!(arena.begin + 1, *arena.current.borrow());
            let mr = arena.fixed_new(43);
            assert_eq!(arena.begin + 8, *arena.current.borrow());
        }
    }
}

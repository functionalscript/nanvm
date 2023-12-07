use core::{
    alloc::Layout,
    sync::atomic::{Ordering, AtomicUsize},
};

use super::{
    atomic_counter::AtomicCounter,
    field_layout::FieldLayout,
    global::{Global, GLOBAL},
    manager::Manager,
};

struct Local {
    counter: AtomicUsize,
    size: AtomicUsize,
}

impl Default for Local {
    #[inline(always)]
    fn default() -> Self {
        Self {
            counter: AtomicUsize::new(0),
            size: AtomicUsize::new(0),
        }
    }
}

type Header = *const Local;

type HeaderLayout = FieldLayout<Header, u8>;

impl Local {
    fn layout(block_layout: Layout) -> (HeaderLayout, Layout) {
        let header_layout = HeaderLayout::align_to(block_layout.align());
        let layout = header_layout.layout(block_layout.size());
        (header_layout, layout)
    }
}

impl Manager for &Local {
    type BlockHeader = AtomicCounter;
    unsafe fn alloc(self, block_layout: Layout) -> *mut u8 {
        let (header_layout, layout) = Local::layout(block_layout);
        self.counter.fetch_add(1, Ordering::Relaxed);
        self.size.fetch_add(layout.size(), Ordering::Relaxed);
        let p = GLOBAL.alloc(layout) as *mut Header;
        *p = self;
        header_layout.to_adjacent(&mut *p) as *mut u8
    }
    unsafe fn dealloc(block_p: *mut u8, block_layout: Layout) {
        let (header_layout, layout) = Local::layout(block_layout);
        let p = header_layout.from_adjancent_mut(&mut *block_p);
        {
            let local = &**p;
            local.counter.fetch_sub(1, Ordering::Relaxed);
            local.size.fetch_sub(layout.size(), Ordering::Relaxed);
        }
        Global::dealloc(p as *mut u8, layout);
    }
}

#[cfg(test)]
mod test {
    use core::{sync::atomic::Ordering, mem::size_of};

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::manager::Manager;

    use super::Local;

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        let local = Local::default();
        {
            let mr = local.fixed_new(42);
            assert_eq!(mr.0, 42);
            assert_eq!(local.counter.load(Ordering::Relaxed), 1);
            assert_eq!(local.size.load(Ordering::Relaxed), size_of::<usize>() + size_of::<isize>() + size_of::<i32>());
        }
        assert_eq!(local.counter.load(Ordering::Relaxed), 0);
        assert_eq!(local.size.load(Ordering::Relaxed), 0);
    }
}

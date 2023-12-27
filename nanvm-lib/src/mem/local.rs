use core::{alloc::Layout, cell::Cell, sync::atomic::AtomicIsize};

use crate::common::default::default;

use super::{
    block_header::BlockHeader,
    field_layout::FieldLayout,
    global::{Global, GLOBAL},
    manager::{Dealloc, Manager},
    ref_counter_update::RefCounterUpdate,
};

#[derive(Debug)]
pub struct Local {
    counter: Cell<isize>,
    size: Cell<usize>,
}

impl Default for Local {
    #[inline(always)]
    fn default() -> Self {
        Self {
            counter: default(),
            size: default(),
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

impl Dealloc for &Local {
    type BlockHeader = AtomicIsize;
    #[inline(always)]
    unsafe fn dealloc(block_p: *mut u8, block_layout: Layout) {
        let (header_layout, layout) = Local::layout(block_layout);
        let p = header_layout.from_adjacent_mut(&mut *block_p);
        {
            let local = &**p;
            local.counter.ref_counter_update(RefCounterUpdate::Release);
            local.size.set(local.size.get() - layout.size());
        }
        Global::dealloc(p as *mut u8, layout);
    }
}

impl Manager for &Local {
    type Dealloc = Self;
    unsafe fn alloc(self, block_layout: Layout) -> *mut u8 {
        let (header_layout, layout) = Local::layout(block_layout);
        self.counter.ref_counter_update(RefCounterUpdate::AddRef);
        self.size.set(self.size.get() + layout.size());
        let p = GLOBAL.alloc(layout) as *mut Header;
        *p = self;
        header_layout.to_adjacent(&mut *p) as *mut u8
    }
}

#[cfg(test)]
mod test {
    use core::{mem::size_of, ops::Deref};

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{flexible_array::header::FlexibleArrayHeader, manager::Manager};

    use super::Local;

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        const SIZE: usize = size_of::<usize>() + size_of::<isize>() + size_of::<i32>();
        let local = Local::default();
        {
            let mr = local.fixed_new(42);
            assert_eq!(mr.0, 42);
            assert_eq!(local.counter.get(), 1);
            assert_eq!(local.size.get(), SIZE);
            let r = mr.to_ref();
            assert_eq!(r.0, 42);
            let mr = r.try_to_mut_ref().unwrap();
            assert_eq!(mr.0, 42);
            let r = mr.to_ref();
            let r1 = r.clone();
            let r2 = r1.try_to_mut_ref().unwrap_err();
            let mr = local.fixed_new(44);
            assert_eq!(mr.0, 44);
            assert_eq!(local.counter.get(), 2);
            assert_eq!(local.size.get(), SIZE << 1);
            let mut mr2 = local.flexible_array_new([1, 2, 3].into_iter());
            assert_eq!(mr2.items_mut(), &[1, 2, 3]);
            assert_eq!(local.counter.get(), 3);
            assert_eq!(
                local.size.get(),
                (SIZE << 1)
                    + size_of::<usize>()
                    + size_of::<isize>()
                    + size_of::<usize>()
                    + size_of::<[i32; 3]>()
            );
            let r3 = mr2.to_ref();
            assert_eq!(r3.header.len(), 3);
            assert_eq!(r3.items(), &[1, 2, 3]);
            {
                let r4 = r3.clone();
                let r5 = r4.deref();
                // drop(r4);
                assert_eq!(r5.header.len(), 3);
                assert_eq!(r5.items(), &[1, 2, 3]);
                let _r6 = &*r4;
                assert_eq!(r5.header.len(), 3);
                assert_eq!(r5.items(), &[1, 2, 3]);
            }
            drop(r2);
            assert_eq!(local.counter.get(), 3);
            drop(r);
            assert_eq!(local.counter.get(), 2);
            assert_eq!(
                local.size.get(),
                SIZE + size_of::<usize>()
                    + size_of::<isize>()
                    + size_of::<usize>()
                    + size_of::<[i32; 3]>()
            );
        }
        assert_eq!(local.counter.get(), 0);
        assert_eq!(local.size.get(), 0);
    }
}

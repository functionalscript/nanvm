// Flexible Array Structure
// https://en.wikipedia.org/wiki/Flexible_array_member

use std::{
    alloc::Layout,
    marker::PhantomData,
    mem::{align_of, size_of},
};

use crate::common::usize::max;

pub struct FasLayout<H, I> {
    align: usize,
    header_size: usize,
    item_size: usize,
    _p: PhantomData<(H, I)>,
}

impl<H, I> FasLayout<H, I> {
    pub const fn new() -> Self {
        let i_align = align_of::<I>();
        Self {
            align: max(align_of::<H>(), i_align),
            header_size: {
                let mask = i_align - 1;
                (size_of::<H>() + mask) & !mask
            },
            item_size: size_of::<I>(),
            _p: PhantomData,
        }
    }
    const fn offset(&self, i: usize) -> usize {
        self.header_size + self.item_size * i
    }
    pub const fn layout(&self, size: usize) -> Layout {
        unsafe { Layout::from_size_align_unchecked(self.offset(size), self.align) }
    }
    pub fn get(&self, p: &mut H, i: usize) -> &mut I {
        unsafe {
            let p = p as *mut H as *mut u8;
            let p = p.add(self.offset(i));
            &mut *(p as *mut I)
        }
    }
}

#[cfg(test)]
mod test {
    use super::FasLayout;

    const L88: FasLayout<u8, u8> = FasLayout::new();
    const _: () = assert!(L88.align == 1);
    const _: () = assert!(L88.header_size == 1);
    const _: () = assert!(L88.item_size == 1);
    const _: () = assert!(L88.layout(0).size() == 1);
    const _: () = assert!(L88.layout(0).align() == 1);
    const _: () = assert!(L88.layout(1).size() == 2);

    const L816: FasLayout<u8, u16> = FasLayout::new();
    const _: () = assert!(L816.align == 2);
    const _: () = assert!(L816.header_size == 2);
    const _: () = assert!(L816.item_size == 2);
    const _: () = assert!(L816.layout(0).size() == 2);
    const _: () = assert!(L816.layout(0).align() == 2);
    const _: () = assert!(L816.layout(1).size() == 4);

    const L168: FasLayout<u16, u8> = FasLayout::new();
    const _: () = assert!(L168.align == 2);
    const _: () = assert!(L168.header_size == 2);
    const _: () = assert!(L168.item_size == 1);
    const _: () = assert!(L168.layout(0).size() == 2);
    const _: () = assert!(L168.layout(0).align() == 2);
    const _: () = assert!(L168.layout(1).size() == 3);
}

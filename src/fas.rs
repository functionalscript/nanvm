// Flexible Array Structure
// https://en.wikipedia.org/wiki/Flexible_array_member

use std::{
    alloc::Layout,
    mem::{align_of, size_of},
};

use crate::usize::max;

pub struct FasLayout {
    size: usize,
    align: usize,
    item_size: usize,
}

impl FasLayout {
    pub const fn new<T, I>() -> Self {
        let i_align = align_of::<I>();
        let c = Layout::new::<T>();
        Self {
            align: max(c.align(), i_align),
            size: (c.size() + i_align - 1) / i_align * i_align,
            item_size: size_of::<I>(),
        }
    }
    pub const fn layout(&self, size: usize) -> Layout {
        unsafe { Layout::from_size_align_unchecked(self.size + self.item_size * size, self.align) }
    }
}

#[cfg(test)]
mod test {
    use super::FasLayout;

    const L88: FasLayout = FasLayout::new::<u8, u8>();
    const _: () = assert!(L88.align == 1);
    const _: () = assert!(L88.size == 1);
    const _: () = assert!(L88.item_size == 1);
    const _: () = assert!(L88.layout(0).size() == 1);
    const _: () = assert!(L88.layout(0).align() == 1);
    const _: () = assert!(L88.layout(1).size() == 2);

    const L816: FasLayout = FasLayout::new::<u8, u16>();
    const _: () = assert!(L816.align == 2);
    const _: () = assert!(L816.size == 2);
    const _: () = assert!(L816.item_size == 2);
    const _: () = assert!(L816.layout(0).size() == 2);
    const _: () = assert!(L816.layout(0).align() == 2);
    const _: () = assert!(L816.layout(1).size() == 4);

    const L168: FasLayout = FasLayout::new::<u16, u8>();
    const _: () = assert!(L168.align == 2);
    const _: () = assert!(L168.size == 2);
    const _: () = assert!(L168.item_size == 1);
    const _: () = assert!(L168.layout(0).size() == 2);
    const _: () = assert!(L168.layout(0).align() == 2);
    const _: () = assert!(L168.layout(1).size() == 3);
}

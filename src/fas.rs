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
        let align = max(c.align(), i_align);
        let size = (c.size() + i_align - 1) / i_align * i_align;
        Self {
            align,
            size,
            item_size: size_of::<I>(),
        }
    }
    pub const fn layout(&self, size: usize) -> Layout {
        unsafe { Layout::from_size_align_unchecked(self.size + self.item_size * size, self.align) }
    }
}

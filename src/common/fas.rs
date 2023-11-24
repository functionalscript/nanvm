// Flexible Array Structure
// https://en.wikipedia.org/wiki/Flexible_array_member

use core::{
    alloc::Layout,
    marker::PhantomData,
    mem::{align_of, size_of},
    slice::from_raw_parts_mut,
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
    const fn size(&self, len: usize) -> usize {
        self.header_size + self.item_size * len
    }
    pub const fn layout(&self, len: usize) -> Layout {
        unsafe { Layout::from_size_align_unchecked(self.size(len), self.align) }
    }
    pub fn get_mut(&self, header: &mut H, len: usize) -> &mut [I] {
        unsafe {
            let p = header as *mut H as *mut u8;
            let p = p.add(self.header_size);
            from_raw_parts_mut(&mut *(p as *mut I), len)
        }
    }
}

#[cfg(test)]
mod test {
    use std::mem::size_of;

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

    #[test]
    fn test_large_struct() {
        struct LargeStruct([u8; 1024]);
        const LAYOUT: FasLayout<LargeStruct, u8> = FasLayout::new();
        assert_eq!(LAYOUT.header_size, 1024);
        assert_eq!(LAYOUT.item_size, 1);
        assert_eq!(LAYOUT.layout(10).size(), 1034); // 1024 for header + 10 for items
    }

    #[repr(align(128))]
    struct Aligned128(u8);

    #[test]
    fn test_unusual_alignment() {
        const LAYOUT: FasLayout<u8, Aligned128> = FasLayout::new();
        assert_eq!(LAYOUT.align, 128);
        assert_eq!(LAYOUT.header_size, 128); // Aligns to 128
        assert_eq!(LAYOUT.item_size, size_of::<Aligned128>());
    }

    #[test]
    fn test_zero_sized_type() {
        struct ZeroSizedType;
        const LAYOUT: FasLayout<ZeroSizedType, u8> = FasLayout::new();
        assert_eq!(LAYOUT.header_size, 0);
        assert_eq!(LAYOUT.layout(10).size(), 10); // Only space for items
    }

    #[test]
    fn test_different_combinations() {
        const LAYOUT1: FasLayout<u16, u32> = FasLayout::new();
        assert_eq!(LAYOUT1.header_size, 4); // Aligns to 4 (u32 alignment)
        assert_eq!(LAYOUT1.layout(2).size(), 12); // 4 for header + 8 for items

        const LAYOUT2: FasLayout<u64, u8> = FasLayout::new();
        assert_eq!(LAYOUT2.header_size, 8); // Aligns to 8 (u64 alignment)
        assert_eq!(LAYOUT2.layout(3).size(), 11); // 8 for header + 3 for items
    }
}

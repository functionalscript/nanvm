use core::{
    alloc::Layout,
    marker::PhantomData,
    mem::{align_of, size_of},
};

use crate::common::usize::max;

pub struct FieldLayout<T, A> {
    pub align: usize,
    pub size: usize,
    _0: PhantomData<(T, A)>,
}

pub const fn align_to(i: usize, align: usize) -> usize {
    let mask = align - 1;
    (i + mask) & !mask
}

impl<T, A> FieldLayout<T, A> {
    pub const fn align_to(adjacent_align: usize) -> FieldLayout<T, A> {
        assert!(adjacent_align.is_power_of_two());
        assert!(adjacent_align >= align_of::<A>());
        FieldLayout {
            align: max(align_of::<T>(), adjacent_align),
            size: align_to(size_of::<T>(), adjacent_align),
            _0: PhantomData,
        }
    }
    #[inline(always)]
    pub const unsafe fn to_adjacent(&self, r: &T) -> *const A {
        unsafe { (r as *const T).byte_add(self.size) as *const _ }
    }
    #[inline(always)]
    pub unsafe fn to_adjacent_mut(&self, r: &mut T) -> *mut A {
        unsafe { (r as *mut T).byte_add(self.size) as *mut _ }
    }
    #[inline(always)]
    pub unsafe fn from_adjacent_mut(&self, r: &mut A) -> *mut T {
        unsafe { (r as *mut A).byte_sub(self.size) as *mut _ }
    }
    #[inline(always)]
    pub const fn layout(&self, a_size: usize) -> Layout {
        unsafe { Layout::from_size_align_unchecked(self.size + a_size, self.align) }
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::FieldLayout;

    const _A_4_1: FieldLayout<u32, u8> = FieldLayout::align_to(1);
    const _: () = assert!(_A_4_1.align == 4 && _A_4_1.size == 4);

    const _A_4_8: FieldLayout<u32, u8> = FieldLayout::align_to(8);
    const _: () = assert!(_A_4_8.align == 8 && _A_4_8.size == 8);

    const _A_1_4: FieldLayout<[u8; 3], u8> = FieldLayout::align_to(4);
    const _: () = assert!(_A_1_4.align == 4 && _A_1_4.size == 4);

    #[test]
    #[should_panic]
    #[wasm_bindgen_test]
    fn test_invalid_align1() {
        FieldLayout::<u32, u8>::align_to(3);
    }

    #[test]
    #[should_panic]
    #[wasm_bindgen_test]
    fn test_invalid_align2() {
        FieldLayout::<u32, u16>::align_to(1);
    }
}

use core::{
    marker::PhantomData,
    mem::{align_of, size_of},
};

use crate::common::usize::max;

pub struct FieldLayout<T, A> {
    pub align: usize,
    pub size: usize,
    _0: PhantomData<(T, A)>,
}

impl<T, A> FieldLayout<T, A> {
    pub const fn align_to(adjacent_align: usize) -> FieldLayout<T, A> {
        FieldLayout {
            align: max(align_of::<T>(), adjacent_align),
            size: {
                let mask = adjacent_align - 1;
                (size_of::<T>() + mask) & !mask
            },
            _0: PhantomData,
        }
    }
    pub fn to_adjacent(&self, r: &mut T) -> *mut A {
        unsafe { (r as *mut T as *mut u8).add(self.size) as *mut A }
    }
}

use core::{
    marker::PhantomData,
    mem::{align_of, size_of},
};

use crate::common::usize::max;

pub struct TypedLayout<T, A> {
    pub align: usize,
    pub size: usize,
    _0: PhantomData<(T, A)>,
}

impl<T, A> TypedLayout<T, A> {
    pub const fn align_to(adjacent_align: usize) -> TypedLayout<T, A> {
        TypedLayout {
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

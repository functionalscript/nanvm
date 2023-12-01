use core::{
    marker::PhantomData,
    mem::{align_of, size_of},
};

use crate::common::usize::max;

pub struct TypedLayout<T, N> {
    pub align: usize,
    pub size: usize,
    _0: PhantomData<(T, N)>,
}

impl<T, N> TypedLayout<T, N> {
    pub const fn align_to(n_align: usize) -> TypedLayout<T, N> {
        TypedLayout {
            align: max(align_of::<T>(), n_align),
            size: {
                let mask = n_align - 1;
                (size_of::<T>() + mask) & !mask
            },
            _0: PhantomData,
        }
    }
    pub fn to_end(&self, r: &mut T) -> *mut N {
        unsafe { (r as *mut T as *mut u8).add(self.size) as *mut N }
    }
}

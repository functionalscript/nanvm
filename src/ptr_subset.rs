use std::marker::PhantomData;

use crate::{
    bit_subset64::BitSubset64,
    const_assert::const_assert,
    container::{Containable, Container},
    value::Value,
};

pub struct PtrSubset<T: Containable>(BitSubset64, PhantomData<T>);

// 49 bits for now
pub const PTR_SUBSET_SUPERPOSITION: u64 = 0x1_FFFF_FFFF_FFFF;

impl<T: Containable> PtrSubset<T> {
    #[inline(always)]
    pub fn update<const ADD: bool>(&self, p: u64) {
        unsafe {
            Container::update::<ADD>(p as *mut Container<T>);
        }
    }
    #[inline(always)]
    pub const fn subset(&self) -> BitSubset64 {
        self.0
    }
    pub const fn new(subset: BitSubset64) -> Self {
        const_assert(subset.superposition() == PTR_SUBSET_SUPERPOSITION);
        Self(subset, PhantomData)
    }
}

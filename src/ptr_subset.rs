use core::marker::PhantomData;

use crate::{
    common::bit_subset64::BitSubset64,
    container::{Container, Info},
};

pub struct PtrSubset<T>(BitSubset64, PhantomData<T>);

// 49 bits for now
pub const PTR_SUBSET_SUPERPOSITION: u64 = 0x1_FFFF_FFFF_FFFF;

impl<T: Info> PtrSubset<T> {
    #[inline(always)]
    pub fn dealloc(&self, p: u64) {
        Container::dealloc(p as *mut Container<T>);
    }
    #[inline(always)]
    pub const fn subset(&self) -> BitSubset64 {
        self.0
    }
    pub const fn new(subset: BitSubset64) -> Self {
        assert!(subset.superposition() == PTR_SUBSET_SUPERPOSITION);
        Self(subset, PhantomData)
    }
}

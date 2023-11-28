use std::mem::forget;

use crate::{
    common::bit_subset64::BitSubset64,
    container::{Container, ContainerRef, Info},
};

pub trait Tag {
    const SUBSET: BitSubset64;
    unsafe fn move_to_unknown_superposition(self) -> u64;
    unsafe fn from_unknown_superposition(u: u64) -> Self;
}

pub trait TagPtr: Info {
    const PTR_SUBSET: BitSubset64;
}

impl<T: TagPtr> Tag for ContainerRef<T> {
    const SUBSET: BitSubset64 = T::PTR_SUBSET;
    #[inline(always)]
    unsafe fn move_to_unknown_superposition(self) -> u64 {
        self.move_to_ref_internal() as u64
    }
    #[inline(always)]
    unsafe fn from_unknown_superposition(u: u64) -> Self {
        Self::from_ref_internal(u as *mut Container<T>)
    }
}

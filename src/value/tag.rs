use std::mem::forget;

use crate::{
    common::bit_subset64::BitSubset64,
    container::{Container, ContainerRef, Info},
};

pub trait Tag {
    const SUBSET: BitSubset64;
    fn to_unknown_raw(self) -> u64;
    fn from_unknown_raw(u: u64) -> Self;
}

pub trait TagPtr: Info {
    const PTR_SUBSET: BitSubset64;
}

impl<T: TagPtr> Tag for ContainerRef<T> {
    const SUBSET: BitSubset64 = T::PTR_SUBSET;
    #[inline(always)]
    fn to_unknown_raw(self) -> u64 {
        let p: *mut Container<T> = *self.get();
        forget(self);
        p as u64
    }
    #[inline(always)]
    fn from_unknown_raw(u: u64) -> Self {
        todo!();
    }
}

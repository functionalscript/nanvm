use crate::{
    common::bit_subset64::BitSubset64,
    container::{Container, Info, Rc},
};

pub trait Tag {
    const SUBSET: BitSubset64;
    unsafe fn move_to_superposition(self) -> u64;
    unsafe fn from_superposition(u: u64) -> Self;
}

pub trait TagRc: Info {
    const PTR_SUBSET: BitSubset64;
}

impl<T: TagRc> Tag for Rc<T> {
    const SUBSET: BitSubset64 = T::PTR_SUBSET;
    #[inline(always)]
    unsafe fn move_to_superposition(self) -> u64 {
        self.move_to_internal() as u64
    }
    #[inline(always)]
    unsafe fn from_superposition(u: u64) -> Self {
        Self::from_internal(u as *mut Container<T>)
    }
}

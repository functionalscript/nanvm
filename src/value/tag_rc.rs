use crate::{
    common::bit_subset64::BitSubset64,
    container::{Base, Container, Info, Rc},
};

use super::{extension::PTR_SUBSET_SUPERPOSITION, tag::Tag};

pub trait TagRc: Info {
    const RC_SUBSET: BitSubset64;
    const _0: () = assert!(Self::RC_SUBSET.superposition() == PTR_SUBSET_SUPERPOSITION);
    unsafe fn dealloc(p: *mut Base) {
        Container::dealloc(p as *mut Container<Self>);
    }
}

impl<T: TagRc> Tag for Rc<T> {
    const SUBSET: BitSubset64 = T::RC_SUBSET;
    #[inline(always)]
    unsafe fn move_to_superposition(self) -> u64 {
        self.move_to_internal() as u64
    }
    #[inline(always)]
    unsafe fn from_superposition(u: u64) -> Self {
        Self::from_internal(u as *mut Container<T>)
    }
}

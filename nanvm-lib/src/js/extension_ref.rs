use crate::{
    common::bit_subset64::BitSubset64,
    mem::{block::Block, manager::Dealloc, object::Object, ref_::Ref},
};

use super::{bitset::RC_SUBSET_SUPERPOSITION, extension::Extension};

pub trait ExtensionRef: Object {
    const REF_SUBSET: BitSubset64;
    const _0: () = assert!(Self::REF_SUBSET.superposition() == RC_SUBSET_SUPERPOSITION);
}

impl<T: ExtensionRef, D: Dealloc> Extension for Ref<T, D> {
    const SUBSET: BitSubset64 = T::REF_SUBSET;
    #[inline(always)]
    unsafe fn move_to_superposition(self) -> u64 {
        self.move_to_internal() as u64
    }
    #[inline(always)]
    unsafe fn from_superposition(u: u64) -> Self {
        Self::new(u as *mut Block<T, D>)
    }
}

use crate::common::bit_subset64::BitSubset64;

use super::{extension::BOOL, tag::Tag};

impl Tag for bool {
    const SUBSET: BitSubset64 = BOOL;
    #[inline(always)]
    unsafe fn move_to_superposition(self) -> u64 {
        self as u64
    }
    #[inline(always)]
    unsafe fn from_superposition(u: u64) -> Self {
        u != 0
    }
}

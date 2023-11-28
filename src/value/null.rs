use crate::common::bit_subset64::BitSubset64;

use super::{extension::NULL, tag::Tag};

pub struct Null();

impl Tag for Null {
    const SUBSET: BitSubset64 = NULL;
    #[inline(always)]
    unsafe fn move_to_superposition(self) -> u64 {
        0
    }
    #[inline(always)]
    unsafe fn from_superposition(_: u64) -> Self {
        Self()
    }
}

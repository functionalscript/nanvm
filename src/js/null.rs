use crate::common::bit_subset64::BitSubset64;

use super::{bitset::NULL, extension::Extension};

pub struct Null();

impl Extension for Null {
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

use crate::common::bit_subset64::BitSubset64;

use super::{extension::BOOL, tag::Tag};

impl Tag for bool {
    const SUBSET: BitSubset64 = BOOL;
    #[inline(always)]
    unsafe fn move_to_unknown_raw(self) -> u64 {
        self as u64
    }
    #[inline(always)]
    unsafe fn from_unknown_raw(u: u64) -> Self {
        u != 0
    }
}

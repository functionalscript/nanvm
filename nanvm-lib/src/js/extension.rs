use crate::common::bit_subset64::BitSubset64;

pub trait Extension {
    const SUBSET: BitSubset64;
    unsafe fn move_to_superposition(self) -> u64;
    unsafe fn from_superposition(u: u64) -> Self;
}

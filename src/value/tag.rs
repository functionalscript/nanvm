use crate::common::bit_subset64::BitSubset64;

pub trait Tag {
    const SUBSET: BitSubset64;
    fn to_unknown_raw(self) -> u64;
    fn from_unknown_raw(u: u64) -> Self;
}

use crate::common::bit_subset64::BitSubset64;

pub trait Tag {
    const SUBSET: BitSubset64;
    fn to_unknown(self) -> u64;
    fn from_unknown(u: u64) -> Self;
}

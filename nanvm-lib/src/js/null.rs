use crate::common::{bit_subset64::BitSubset64, cast::Cast};

use super::{bitset::NULL, value_cast::ValueCast};

pub struct Null();

impl Cast<u64> for Null {
    #[inline(always)]
    fn cast(self) -> u64 {
        0
    }
}

impl Cast<Null> for u64 {
    #[inline(always)]
    fn cast(self) -> Null {
        Null()
    }
}

impl ValueCast for Null {
    const SUBSET: BitSubset64<Null> = NULL;
}

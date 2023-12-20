use crate::common::bit_subset64::{BitSubset64, Cast};

pub trait Extension: Sized + Cast<u64>
where
    u64: Cast<Self>,
{
    const SUBSET: BitSubset64<Self>;
}

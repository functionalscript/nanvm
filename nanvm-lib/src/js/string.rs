use crate::{
    common::bit_subset64::BitSubset64,
    mem::{block::Block, flexible_array::FlexibleArray, manager::Dealloc},
};

use super::{bitset::STRING, ref_cast::RefCast};

pub type StringHeader = FlexibleArray<u16>;

impl<D: Dealloc> RefCast<D> for StringHeader {
    const REF_SUBSET: BitSubset64<*const Block<StringHeader, D>> = STRING.cast();
}

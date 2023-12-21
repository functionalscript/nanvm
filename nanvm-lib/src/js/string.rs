use crate::{
    common::bit_subset64::BitSubset64,
    mem::{flexible_array::FlexibleArray, manager::Dealloc},
};

use super::{bitset::STRING, extension_ref::ExtensionRef};

pub type StringHeader = FlexibleArray<u16>;

impl<D: Dealloc> ExtensionRef<D> for StringHeader {
    const REF_SUBSET: BitSubset64 = STRING;
}

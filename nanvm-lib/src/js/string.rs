use crate::{common::bit_subset64::BitSubset64, mem::flexible_array::FlexibleArray};

use super::{bitset::STRING, extension_ref::ExtensionRef};

pub type StringHeader = FlexibleArray<u16>;

impl ExtensionRef for StringHeader {
    const REF_SUBSET: BitSubset64 = STRING;
}

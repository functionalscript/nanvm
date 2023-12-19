use crate::{common::bit_subset64::BitSubset64, mem::flexible_array::FlexibleArray};

use super::{bitset::STRING, extension_ref::ExtensionRef};

pub type StringHeader2 = FlexibleArray<u16>;

impl ExtensionRef for StringHeader2 {
    const REF_SUBSET: BitSubset64 = STRING;
}

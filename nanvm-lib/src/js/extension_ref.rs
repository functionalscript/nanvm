use crate::{common::bit_subset64::BitSubset64, mem::object::Object};

use super::bitset::REF_SUBSET_SUPERPOSITION;

pub trait ExtensionRef: Object {
    const REF_SUBSET: BitSubset64;
    const _0: () = assert!(Self::REF_SUBSET.superposition() == REF_SUBSET_SUPERPOSITION);
}

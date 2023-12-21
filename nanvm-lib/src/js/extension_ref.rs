use crate::{
    common::bit_subset64::BitSubset64,
    mem::{block::Block, manager::Dealloc, object::Object},
};

use super::bitset::REF_SUBSET_SUPERPOSITION;

pub trait ExtensionRef<D: Dealloc>: Object {
    const REF_SUBSET: BitSubset64<*const Block<Self, D>>;
    const _0: () = assert!(Self::REF_SUBSET.superposition() == REF_SUBSET_SUPERPOSITION);
}

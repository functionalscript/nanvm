use crate::{
    common::bit_subset64::BitSubset64,
    ptr_subset::PtrSubset,
    value::{object::ObjectHeader, string::StringHeader},
};

pub const EXTENSION: BitSubset64 = BitSubset64::from_tag(0xFFF8_0000_0000_0000);

const EXTENSION_SPLIT: (BitSubset64, BitSubset64) = EXTENSION.split(0x0004_0000_0000_0000);

pub const BOOL: BitSubset64 = EXTENSION_SPLIT.0;
pub const PTR: BitSubset64 = EXTENSION_SPLIT.1;

const PTR_SPLIT: (BitSubset64, BitSubset64) = PTR.split(0x0002_0000_0000_0000);

pub const STRING: PtrSubset<StringHeader> = PTR_SPLIT.0.ptr_subset();
const STRING_TAG: u64 = STRING.subset().tag;
pub const OBJECT: PtrSubset<ObjectHeader> = PTR_SPLIT.1.ptr_subset();
const OBJECT_TAG: u64 = OBJECT.subset().tag;

pub const FALSE: u64 = BOOL.tag;
pub const TRUE: u64 = BOOL.tag | 1;

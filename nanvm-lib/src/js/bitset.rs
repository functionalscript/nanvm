use crate::common::bit_subset64::BitSubset64;

// EXTENSION: 1111_1111_1111_1.X.X

pub const EXTENSION: BitSubset64 = BitSubset64::from_tag(0xFFF8_0000_0000_0000);

const EXTENSION_SPLIT: (BitSubset64, BitSubset64) = EXTENSION.split(0x0004_0000_0000_0000);

pub const SIMPLE: BitSubset64 = EXTENSION_SPLIT.0;
pub const REF: BitSubset64 = EXTENSION_SPLIT.1;

// SIMPLE: 1111_1111_1111_1.0.X

pub const SIMPLE_SPLIT: (BitSubset64, BitSubset64) = SIMPLE.split(0x0002_0000_0000_0000);

// BOOL: 1111_1111_1111_1.0.0

pub const BOOL: BitSubset64 = SIMPLE_SPLIT.0;

pub const FALSE: u64 = BOOL.from_value(false as u64);
pub const TRUE: u64 = BOOL.from_value(true as u64);

// NULL: 1111_1111_1111_1.0.1

pub const NULL: BitSubset64 = SIMPLE_SPLIT.1;

// RC: 1111_1111_1111_1.1.X

// 49 bits for now
pub const REF_SUBSET_SUPERPOSITION: u64 = 0x1_FFFF_FFFF_FFFF;

const REF_SPLIT: (BitSubset64, BitSubset64) = REF.split(0x0002_0000_0000_0000);

// STRING: 1111_1111_1111_1.1.0

pub const STRING: BitSubset64 = REF_SPLIT.0;

// OBJECT: 1111_1111_1111_1.1.1

pub const OBJECT: BitSubset64 = REF_SPLIT.1;

#[cfg(test)]
mod test {
    use crate::js::{
        bitset::{BOOL, EXTENSION, FALSE, TRUE},
        number::test::NAN,
    };

    const _: () = assert!(BOOL.has(FALSE));
    const _: () = assert!(BOOL.has(TRUE));
    const _: () = assert!(!BOOL.has(0));
    const _: () = assert!(!BOOL.has(NAN));
    const _: () = assert!(BOOL.has(EXTENSION.mask));
}

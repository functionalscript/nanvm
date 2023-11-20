use std::marker::PhantomData;

use crate::{const_assert::const_assert, container::Clean, ptr_subset::PtrSubset};

/// A bit subset of `u64`.
///
/// This structure represents a subset of bits within a 64-bit unsigned integer,
/// defined by a specific pattern where each bit is either `0`, `1`, or `S`.
///
/// - `0` means the bit is always `0`. These bits contribute to both the tag and the mask.
/// - `1` means the bit is always `1`. These bits also contribute to both the tag and the mask.
/// - `S` represents a superposition state, meaning the bit can be either `0` or `1`.
///
/// The cardinality of the set is calculated as `2^(superposition.count_ones())`, representing
/// the number of unique combinations possible within the superposition bits.
///
/// The following table summarizes how each field is derived:
///
/// |Property      | 0 | 1 | S | Description        |
/// |--------------|---|---|---|--------------------|
/// | union        | 0 | 1 | 1 | items.reduce(or)   |
/// | tag          | 0 | 1 | 0 | items.reduce(and)  |
/// | superposition| 0 | 0 | 1 | union ^ tag        |
/// | mask         | 1 | 1 | 0 | !superposition     |
#[derive(Clone, Copy)]
pub struct BitSubset64 {
    /// Represents the intersection of all items in the subset. A pattern of bits
    /// where a `1` in each position indicates that the corresponding bit is consistently `1`
    /// across all items, and a `0` indicates that it is not consistently `1`.
    pub tag: u64,
    /// Identifies the bits that are constant (either `0` or `1`). A `1` in a position
    /// indicates a fixed bit (as per the `tag`), and a `0` indicates a superposition bit.
    pub mask: u64,
}

impl BitSubset64 {
    #[inline(always)]
    pub const fn from_tag_and_mask(tag: u64, mask: u64) -> Self {
        const_assert(mask & tag == tag);
        Self { tag, mask }
    }
    #[inline(always)]
    pub const fn from_tag_and_superposition(tag: u64, sup: u64) -> Self {
        Self::from_tag_and_mask(tag, !sup)
    }
    #[inline(always)]
    pub const fn from_tag_and_union(tag: u64, union: u64) -> Self {
        Self::from_tag_and_superposition(tag, tag ^ union)
    }
    #[inline(always)]
    pub const fn from_tag(tag: u64) -> Self {
        Self::from_tag_and_mask(tag, tag)
    }
    #[inline(always)]
    pub const fn has(self, value: u64) -> bool {
        value & self.mask == self.tag
    }
    #[inline(always)]
    pub const fn union(self) -> u64 {
        self.tag ^ self.superposition()
    }
    #[inline(always)]
    pub const fn superposition(self) -> u64 {
        !self.mask
    }
    #[inline(always)]
    pub const fn or_unchecked(self, b: BitSubset64) -> BitSubset64 {
        BitSubset64::from_tag_and_union(self.tag & b.tag, self.union() | b.union())
    }
    #[inline(always)]
    pub const fn or(self, b: BitSubset64) -> BitSubset64 {
        const_assert(self.superposition() == b.superposition());
        self.or_unchecked(b)
    }
    #[inline(always)]
    pub const fn and(self, b: BitSubset64) -> BitSubset64 {
        BitSubset64::from_tag_and_union(self.tag | b.tag, self.union() & b.union())
    }
    #[inline(always)]
    pub const fn split(self, m: u64) -> (BitSubset64, BitSubset64) {
        const_assert(m != 0);
        const_assert(m & self.mask == 0);
        let mask = self.mask | m;
        (
            BitSubset64::from_tag_and_mask(self.tag, mask),
            BitSubset64::from_tag_and_mask(self.tag | m, mask),
        )
    }
    #[inline(always)]
    pub const fn ptr_subset<T: Clean>(self) -> PtrSubset<T> {
        PtrSubset(self, PhantomData)
    }
}

#[cfg(test)]
mod test {
    use crate::const_assert::const_assert;

    use super::BitSubset64;

    const A: BitSubset64 = BitSubset64::from_tag_and_union(0b010, 0b011);
    const _: () = const_assert(A.superposition() == 0b001);
    const _: () = const_assert(A.tag == 0b010);
    const _: () = const_assert(!A.has(0b000));
    const _: () = const_assert(A.has(0b010));
    const _: () = const_assert(A.has(0b011));

    const AS: (BitSubset64, BitSubset64) = A.split(1);
    const _: () = const_assert(AS.0.tag == 0b010);
    const _: () = const_assert(AS.0.superposition() == 0);
    const _: () = const_assert(AS.1.tag == 0b011);
    const _: () = const_assert(AS.1.superposition() == 0);

    #[test]
    fn test_a() {
        assert_eq!(A.superposition(), 0b001);
        assert_eq!(A.tag, 0b010);
        assert!(!A.has(0b000));
        assert!(A.has(0b010));
        assert!(A.has(0b011));
    }

    const B: BitSubset64 = BitSubset64::from_tag_and_union(0b000110, 0b000111);
    const C: BitSubset64 = BitSubset64::from_tag_and_union(0b010100, 0b011111);
    const UBC: BitSubset64 = B.or_unchecked(C);
    const _: () = const_assert(UBC.superposition() == 0b011011);
    const _: () = const_assert(UBC.tag == 0b000100);
    const _: () = const_assert(UBC.union() == 0b011111);

    const _UBCS: (BitSubset64, BitSubset64) = UBC.split(0b1000);
    const _: () = const_assert(_UBCS.0.superposition() == 0b010011);
    const _: () = const_assert(_UBCS.0.tag == 0b000100);
    const _: () = const_assert(_UBCS.1.tag == 0b001100);

    #[test]
    fn test_ubc() {
        assert_eq!(UBC.superposition(), 0b011011);
        assert_eq!(UBC.tag, 0b000100);
        assert_eq!(UBC.union(), 0b011111);
    }

    #[test]
    #[should_panic]
    fn test_ibc() {
        B.and(C);
    }

    #[test]
    #[should_panic]
    fn test_split_fail() {
        UBC.split(0b100);
    }

    const D: BitSubset64 = BitSubset64::from_tag_and_union(0b00110, 0b00111);
    const E: BitSubset64 = BitSubset64::from_tag_and_union(0b00100, 0b01111);
    const UDE: BitSubset64 = D.or_unchecked(E);
    const _: () = const_assert(UDE.superposition() == 0b01011);
    const _: () = const_assert(UDE.tag == 0b00100);
    const _: () = const_assert(UDE.union() == 0b01111);
    const IDE: BitSubset64 = D.and(E);
    const _: () = const_assert(IDE.superposition() == 0b00001);
    const _: () = const_assert(IDE.tag == 0b00110);
    const _: () = const_assert(IDE.union() == 0b00111);

    #[test]
    fn test_ude() {
        assert_eq!(UDE.superposition(), 0b01011);
        assert_eq!(UDE.tag, 0b00100);
        assert_eq!(UDE.union(), 0b01111);
    }

    #[test]
    fn test_ide() {
        assert_eq!(IDE.superposition(), 0b00001);
        assert_eq!(IDE.tag, 0b00110);
        assert_eq!(IDE.union(), 0b00111);
    }
}

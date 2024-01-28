use core::marker::PhantomData;

use super::cast::Cast;

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
pub struct BitSubset64<T: Cast<u64> = u64>
where
    u64: Cast<T>,
{
    /// Represents the intersection of all items in the subset. A pattern of bits
    /// where a `1` in each position indicates that the corresponding bit is consistently `1`
    /// across all items, and a `0` indicates that it is not consistently `1`.
    pub tag: u64,
    /// Identifies the bits that are constant (either `0` or `1`). A `1` in a position
    /// indicates a fixed bit (as per the `tag`), and a `0` indicates a superposition bit.
    pub mask: u64,
    _0: PhantomData<T>,
}

impl<T: Cast<u64>> Clone for BitSubset64<T>
where
    u64: Cast<T>,
{
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Cast<u64>> Copy for BitSubset64<T> where u64: Cast<T> {}

impl BitSubset64<u64> {
    #[inline(always)]
    pub const fn cast<U: Cast<u64>>(self) -> BitSubset64<U>
    where
        u64: Cast<U>,
    {
        BitSubset64 {
            tag: self.tag,
            mask: self.mask,
            _0: PhantomData,
        }
    }
}

impl<T: Cast<u64>> BitSubset64<T>
where
    u64: Cast<T>,
{
    #[inline(always)]
    pub const fn from_tag_and_mask(tag: u64, mask: u64) -> Self {
        assert!(mask & tag == tag);
        Self {
            tag,
            mask,
            _0: PhantomData,
        }
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
    pub const fn or_unchecked(self, b: Self) -> Self {
        Self::from_tag_and_union(self.tag & b.tag, self.union() | b.union())
    }
    #[inline(always)]
    pub const fn or(self, b: Self) -> Self {
        assert!(self.superposition() == b.superposition());
        self.or_unchecked(b)
    }
    #[inline(always)]
    pub const fn and(self, b: Self) -> Self {
        Self::from_tag_and_union(self.tag | b.tag, self.union() & b.union())
    }
    #[inline(always)]
    pub const fn split(self, sub_mask: u64) -> (Self, Self) {
        // we need at least one bit to distinguish the two subsets.
        assert!(sub_mask != 0);
        // the bit shouldn't be a part of the original set mask.
        assert!(sub_mask & self.mask == 0);
        let mask = self.mask | sub_mask;
        // the subsets should have different tags.
        (
            Self::from_tag_and_mask(self.tag, mask),
            Self::from_tag_and_mask(self.tag | sub_mask, mask),
        )
    }
    #[inline(always)]
    const fn subset_value_to_raw_value(self, set: u64) -> u64 {
        self.superposition() & set
    }
    #[inline(always)]
    pub const fn raw_value_to_subset_value(self, value: u64) -> u64 {
        self.tag | value
    }
    #[inline(always)]
    pub fn typed_value_to_subset_value(self, value: T) -> u64 {
        self.raw_value_to_subset_value(value.cast())
    }
    #[inline(always)]
    pub fn subset_value_to_typed_value(self, set: u64) -> T {
        (self.subset_value_to_raw_value(set)).cast()
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::BitSubset64;

    const A: BitSubset64 = BitSubset64::from_tag_and_union(0b010, 0b011);
    const _: () = assert!(A.superposition() == 0b001);
    const _: () = assert!(A.tag == 0b010);
    const _: () = assert!(!A.has(0b000));
    const _: () = assert!(A.has(0b010));
    const _: () = assert!(A.has(0b011));

    const _AS: (BitSubset64, BitSubset64) = A.split(1);
    const _: () = assert!(_AS.0.tag == 0b010);
    const _: () = assert!(_AS.0.superposition() == 0);
    const _: () = assert!(_AS.1.tag == 0b011);
    const _: () = assert!(_AS.1.superposition() == 0);

    #[test]
    #[wasm_bindgen_test]
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
    const _: () = assert!(UBC.superposition() == 0b011011);
    const _: () = assert!(UBC.tag == 0b000100);
    const _: () = assert!(UBC.union() == 0b011111);

    const _UBCS: (BitSubset64, BitSubset64) = UBC.split(0b1000);
    const _: () = assert!(_UBCS.0.superposition() == 0b010011);
    const _: () = assert!(_UBCS.0.tag == 0b000100);
    const _: () = assert!(_UBCS.1.tag == 0b001100);

    #[test]
    #[wasm_bindgen_test]
    fn test_ubc() {
        assert_eq!(UBC.superposition(), 0b011011);
        assert_eq!(UBC.tag, 0b000100);
        assert_eq!(UBC.union(), 0b011111);
    }

    #[test]
    #[wasm_bindgen_test]
    #[should_panic]
    fn test_ibc() {
        B.and(C);
    }

    #[test]
    #[wasm_bindgen_test]
    #[should_panic]
    fn test_split_fail() {
        UBC.split(0b100);
    }

    const D: BitSubset64 = BitSubset64::from_tag_and_union(0b00110, 0b00111);
    const E: BitSubset64 = BitSubset64::from_tag_and_union(0b00100, 0b01111);
    const UDE: BitSubset64 = D.or_unchecked(E);
    const _: () = assert!(UDE.superposition() == 0b01011);
    const _: () = assert!(UDE.tag == 0b00100);
    const _: () = assert!(UDE.union() == 0b01111);
    const IDE: BitSubset64 = D.and(E);
    const _: () = assert!(IDE.superposition() == 0b00001);
    const _: () = assert!(IDE.tag == 0b00110);
    const _: () = assert!(IDE.union() == 0b00111);

    #[test]
    #[wasm_bindgen_test]
    fn test_ude() {
        assert_eq!(UDE.superposition(), 0b01011);
        assert_eq!(UDE.tag, 0b00100);
        assert_eq!(UDE.union(), 0b01111);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_ide() {
        assert_eq!(IDE.superposition(), 0b00001);
        assert_eq!(IDE.tag, 0b00110);
        assert_eq!(IDE.union(), 0b00111);
    }
}

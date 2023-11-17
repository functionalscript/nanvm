use crate::const_assert::const_assert;

/// A bit subset of `u64`.
///
/// It only works for sets which are defined by a pattern in the form when every bit is either `0`, `1`, or `S`.
/// - `0` means that bit is always `0`. It's a part of a tag and a mask.
/// - `1` means that bit is always `1`. It's a part of a tag and a mask.
/// - `S` means that bit is either `0` or `1`. It's a part of a superposition.
///
/// `cardinality = 2^(superposition.count_ones())`
///
/// |             |0|1|S|                 |
/// |-------------|-|-|-|-----------------|
/// |union        |0|1|1|items.reduce(or) |
/// |tag          |0|1|0|items.reduce(and)|
/// |superposition|0|0|1|union^tag        |
/// |mask         |1|1|0|!superposition   |
#[derive(Clone, Copy)]
pub struct BitSubset64 {
    pub tag: u64,
    pub mask: u64,
}

impl BitSubset64 {
    #[inline(always)]
    pub const fn from_tag_and_mask(tag: u64, mask: u64) -> Self {
        const_assert(mask & tag == tag);
        Self { tag, mask }
    }
    #[inline(always)]
    pub const fn from_tag_and_superposition(tag: u64, superposition: u64) -> Self {
        Self::from_tag_and_mask(tag, !superposition)
    }
    #[inline(always)]
    pub const fn from_tag_and_union(tag: u64, union: u64) -> Self {
        Self::from_tag_and_superposition(tag,tag ^ union)
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
        self.tag ^ !self.mask
    }
    #[inline(always)]
    pub const fn superposition(self) -> u64 {
        !self.mask
    }
    #[inline(always)]
    pub const fn or(self, b: BitSubset64) -> BitSubset64 {
        BitSubset64::from_tag_and_union(self.tag & b.tag, self.union() | b.union())
    }
    #[inline(always)]
    pub const fn and(self, b: BitSubset64) -> BitSubset64 {
        BitSubset64::from_tag_and_union(self.tag | b.tag, self.union() & b.union())
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
    const UBC: BitSubset64 = B.or(C);
    const _: () = const_assert(UBC.superposition() == 0b011011);
    const _: () = const_assert(UBC.tag == 0b000100);
    const _: () = const_assert(UBC.union() == 0b011111);

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

    const D: BitSubset64 = BitSubset64::from_tag_and_union(0b00110, 0b00111);
    const E: BitSubset64 = BitSubset64::from_tag_and_union(0b00100, 0b01111);
    const UDE: BitSubset64 = D.or(E);
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

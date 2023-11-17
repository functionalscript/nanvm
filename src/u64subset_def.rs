use crate::const_assert::const_assert;

/// A definition of a subset of `u64`.
///
/// It only works for sets which defines by a pattern in the form of every bit is either `0`, `1`, or `S`.
/// - `0` means that bit is always `0`. It's a part of a tag and a mask. This bit is constant and doesn't carry information.
/// - `1` means that bit is always `1`. It's a part of a tag and a mask. This bit is constant and doesn't carry information.
/// - `S` means that bit is either `0` or `1`. It's a part of a superposition. This bit carries information.
///
/// |             |0|1|S|
/// |-------------|-|-|-|
/// |union        |0|1|1|
/// |tag          |0|1|0|
/// |superposition|0|0|1|
/// |mask         |1|1|0|
#[derive(Clone, Copy)]
pub struct U64SubsetDef {
    pub mask: u64,
    pub tag: u64,
}

impl U64SubsetDef {
    #[inline(always)]
    pub const fn new(mask: u64, tag: u64) -> Self {
        Self { mask, tag }
    }
    #[inline(always)]
    pub const fn set(all: u64, tag: u64) -> Self {
        const_assert(all & tag == tag);
        Self::new(!(tag ^ all), tag)
    }
    #[inline(always)]
    pub const fn from_tag(tag: u64) -> Self {
        Self::new(tag, tag)
    }
    #[inline(always)]
    pub const fn is(self, value: u64) -> bool {
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
    pub const fn or(self, b: U64SubsetDef) -> U64SubsetDef {
        U64SubsetDef::set(self.union() | b.union(), self.tag & b.tag)
    }
    #[inline(always)]
    pub const fn and(self, b: U64SubsetDef) -> U64SubsetDef {
        U64SubsetDef::set(self.union() & b.union(), self.tag | b.tag)
    }
}

#[cfg(test)]
mod test {
    use crate::const_assert::const_assert;

    use super::U64SubsetDef;

    const A: U64SubsetDef = U64SubsetDef::set(0b011, 0b010);
    const _: () = const_assert(A.superposition() == 0b001);
    const _: () = const_assert(A.tag == 0b010);
    const _: () = const_assert(!A.is(0b000));
    const _: () = const_assert(A.is(0b010));
    const _: () = const_assert(A.is(0b011));

    #[test]
    fn test_a() {
        assert_eq!(A.superposition(), 0b001);
        assert_eq!(A.tag, 0b010);
        assert!(!A.is(0b000));
        assert!(A.is(0b010));
        assert!(A.is(0b011));
    }

    const B: U64SubsetDef = U64SubsetDef::set(0b000111, 0b000110);
    const C: U64SubsetDef = U64SubsetDef::set(0b011111, 0b010100);
    const UBC: U64SubsetDef = B.or(C);
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

    const D: U64SubsetDef = U64SubsetDef::set(0b00111, 0b00110);
    const E: U64SubsetDef = U64SubsetDef::set(0b01111, 0b00100);
    const UDE: U64SubsetDef = D.or(E);
    const _: () = const_assert(UDE.superposition() == 0b01011);
    const _: () = const_assert(UDE.tag == 0b00100);
    const _: () = const_assert(UDE.union() == 0b01111);
    const IDE: U64SubsetDef = D.and(E);
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

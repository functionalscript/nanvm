use crate::const_assert::const_assert;

/// A subset of `u64`.
///
/// It only works for sets in the form of every bit is either `0`, `1`, or `?`.
///
/// |                |0|1|?|
/// |----------------|-|-|-|
/// |bit_union       |0|1|1|
/// |bit_intersection|0|1|0|
/// |superposition   |0|0|1|
/// |mask            |1|1|0|
#[derive(Clone, Copy)]
pub struct U64Subset {
    pub mask: u64,
    pub flag: u64,
}

impl U64Subset {
    #[inline(always)]
    pub const fn new(mask: u64, flag: u64) -> Self {
        Self { mask, flag }
    }
    #[inline(always)]
    pub const fn set(bit_union: u64, bit_intersection: u64) -> Self {
        const_assert(bit_union & bit_intersection == bit_intersection);
        Self::new(!(bit_intersection ^ bit_union), bit_intersection)
    }
    #[inline(always)]
    pub const fn from_mask(mask: u64) -> Self {
        Self::new(mask, mask)
    }
    #[inline(always)]
    pub const fn is(self, value: u64) -> bool {
        value & self.mask == self.flag
    }
    #[inline(always)]
    pub const fn bit_union(self) -> u64 {
        self.flag ^ !self.mask
    }
    #[inline(always)]
    pub const fn superposition(self) -> u64 {
        !self.mask
    }
    #[inline(always)]
    pub const fn union(self, b: U64Subset) -> U64Subset {
        U64Subset::set(self.bit_union() | b.bit_union(), self.flag & b.flag)
    }
    #[inline(always)]
    pub const fn intersection(self, b: U64Subset) -> U64Subset {
        U64Subset::set(self.bit_union() & b.bit_union(), self.flag | b.flag)
    }
}

#[cfg(test)]
mod test {
    use crate::const_assert::const_assert;

    use super::U64Subset;

    const A: U64Subset = U64Subset::set(0b011, 0b010);
    const _: () = const_assert(A.superposition() == 0b001);
    const _: () = const_assert(A.flag == 0b010);
    const _: () = const_assert(!A.is(0b000));
    const _: () = const_assert(A.is(0b010));
    const _: () = const_assert(A.is(0b011));

    #[test]
    fn test_a() {
        assert_eq!(A.superposition(), 0b001);
        assert_eq!(A.flag, 0b010);
        assert!(!A.is(0b000));
        assert!(A.is(0b010));
        assert!(A.is(0b011));
    }

    const B: U64Subset = U64Subset::set(0b000111, 0b000110);
    const C: U64Subset = U64Subset::set(0b011111, 0b010100);
    const UBC: U64Subset = B.union(C);
    const _: () = const_assert(UBC.superposition() == 0b011011);
    const _: () = const_assert(UBC.flag == 0b000100);
    const _: () = const_assert(UBC.bit_union() == 0b011111);

    #[test]
    fn test_ubc() {
        assert_eq!(UBC.superposition(), 0b011011);
        assert_eq!(UBC.flag, 0b000100);
        assert_eq!(UBC.bit_union(), 0b011111);
    }

    #[test]
    #[should_panic]
    fn test_ibc() {
        B.intersection(C);
    }

    const D: U64Subset = U64Subset::set(0b00111, 0b00110);
    const E: U64Subset = U64Subset::set(0b01111, 0b00100);
    const UDE: U64Subset = D.union(E);
    const _: () = const_assert(UDE.superposition() == 0b01011);
    const _: () = const_assert(UDE.flag == 0b00100);
    const _: () = const_assert(UDE.bit_union() == 0b01111);
    const IDE: U64Subset = D.intersection(E);
    const _: () = const_assert(IDE.superposition() == 0b00001);
    const _: () = const_assert(IDE.flag == 0b00110);
    const _: () = const_assert(IDE.bit_union() == 0b00111);

    #[test]
    fn test_ude() {
        assert_eq!(UDE.superposition(), 0b01011);
        assert_eq!(UDE.flag, 0b00100);
        assert_eq!(UDE.bit_union(), 0b01111);
    }

    #[test]
    fn test_ide() {
        assert_eq!(IDE.superposition(), 0b00001);
        assert_eq!(IDE.flag, 0b00110);
        assert_eq!(IDE.bit_union(), 0b00111);
    }
}

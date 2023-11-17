use crate::const_assert::const_assert;

// subset: {010, 011 }
// definition: 01S
// intersection: 010
// union: 011
// superposition = intersection ^ union: 001
// mask = !superposition = 110
// is(v) = (v & mask) == intersection
//
// superposition = !mask: 001
// union = intersection ^ superposition: 011

#[derive(Debug, Clone, Copy)]
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
    pub const fn set(union: u64, intersection: u64) -> Self {
        const_assert(union & intersection == intersection);
        Self::new(!(intersection ^ union), intersection)
    }
    #[inline(always)]
    pub const fn all(mask: u64) -> Self {
        Self::new(mask, mask)
    }
    #[inline(always)]
    pub const fn is(self, value: u64) -> bool {
        value & self.mask == self.flag
    }
    #[inline(always)]
    pub const fn union(self, b: U64Subset) -> U64Subset {
        U64Subset::new(self.mask | b.mask, self.flag | b.flag)
    }
}
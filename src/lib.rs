#[derive(Debug)]
#[repr(transparent)]
struct Value(u64);

#[derive(Debug, Clone, Copy)]
struct Check {
    mask: u64,
    flag: u64,
}

impl Check {
    #[inline(always)]
    const fn some(mask: u64, flag: u64) -> Self {
        Self { mask, flag }
    }
    #[inline(always)]
    const fn all(mask: u64) -> Self {
        Self::some(mask, mask)
    }
    #[inline(always)]
    const fn is(self, value: u64) -> bool {
        (value & self.mask) == self.flag
    }
}

// compatible with `f64`
const INFINITY: u64 = 0x7FF0_0000_0000_0000;
const NAN: u64 = 0x7FF8_0000_0000_0000;
const NEG_INFINITY: u64 = 0xFFF0_0000_0000_0000;

// not compatible with `f64`
const NAF: u64 = 0xFFF8_0000_0000_0000;

const IS_NAF: Check = Check::all(NAF);

const PTR: u64 = NAF | 0x2_0000_0000_0000;

const IS_PTR: Check = Check::all(PTR);

const STR: u64 = NAF | 0x4_0000_0000_0000;

const IS_STR: Check = Check::all(STR);

const STR_PTR: u64 = STR | PTR;

const IS_STR_PTR: Check = Check::all(STR_PTR);

const FALSE: u64 = NAF;
const TRUE: u64 = NAF | 1;

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use super::*;

    #[test]
    fn test_nan() {
        assert_eq!(f64::INFINITY.to_bits(), INFINITY);
        assert_ne!(f64::NAN, f64::NAN);
        assert_eq!(f64::NAN.to_bits(), NAN);
        assert_eq!(f64::NEG_INFINITY.to_bits(), NEG_INFINITY);
    }

    #[test]
    fn test_unsized() {
        let _x: Rc<[u8]> = Rc::new([1, 3]);
        // let _y: Rc<(u8, [u8])> = Rc::new((5, [1, 3]));
        // let r = Vec::default();
        // let n = 4 + 4;
        // let _y: Rc<[u8]> = Rc::new([5; n]);
    }
}

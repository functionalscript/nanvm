use crate::u64subset::U64Subset;

#[derive(Debug)]
#[repr(transparent)]
struct Value(u64);

// compatible with `f64`
const INFINITY: u64 = 0x7FF0_0000_0000_0000;
const NAN: u64 = 0x7FF8_0000_0000_0000;
const NEG_INFINITY: u64 = 0xFFF0_0000_0000_0000;

const EXTENSION: U64Subset = U64Subset::from_mask(0xFFF8_0000_0000_0000);

const PTR: U64Subset = U64Subset::from_mask(EXTENSION.mask | 0x2_0000_0000_0000);

const STR: U64Subset = U64Subset::from_mask(EXTENSION.mask | 0x4_0000_0000_0000);

const STR_PTR: U64Subset = STR.intersection(PTR);

const FALSE: u64 = EXTENSION.mask;
const TRUE: u64 = EXTENSION.mask | 1;

const BOOL: U64Subset = U64Subset::set(TRUE | FALSE, TRUE & FALSE);

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use super::*;
    use crate::const_assert::const_assert;

    const _: () = const_assert(BOOL.is(FALSE));
    const _: () = const_assert(BOOL.is(TRUE));
    const _: () = const_assert(!BOOL.is(0));
    const _: () = const_assert(!BOOL.is(NAN));
    const _: () = const_assert(BOOL.is(EXTENSION.mask));
    const _: () = const_assert(!BOOL.is(EXTENSION.mask | 2));

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

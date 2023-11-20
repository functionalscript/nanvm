use crate::const_assert::const_assert;

// compatible with `f64`
pub const INFINITY: u64 = 0x7FF0_0000_0000_0000;
pub const NAN: u64 = 0x7FF8_0000_0000_0000;
pub const NEG_INFINITY: u64 = 0xFFF0_0000_0000_0000;
pub const E: u64 = 0x7FF0_0000_0000_0000;
pub const EF: u64 = 0x7FFF_FFFF_FFFF_FFFF;

pub const fn is_valid(v: u64) -> bool {
    v & E != E || v & EF == INFINITY || v == NAN
}

pub const fn check(v: u64) {
    const_assert(is_valid(v));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nan() {
        assert_eq!(f64::INFINITY.to_bits(), INFINITY);
        assert_ne!(f64::NAN, f64::NAN);
        assert_eq!(f64::NAN.to_bits(), NAN);
        assert_eq!(f64::NEG_INFINITY.to_bits(), NEG_INFINITY);
    }

    #[test]
    fn test_check() {
        check(0);
        check(1);
        check(INFINITY);
        check(NAN);
        check(NEG_INFINITY);
        assert_eq!((0.0f64 / 0.0).to_bits(), NAN);
    }

    #[test]
    #[should_panic]
    fn test_nan_panic() {
        check(0x7FF0_00F0_0500_0001);
    }

    #[test]
    #[should_panic]
    fn test_nan_panic2() {
        check(0xFFFA_FF96_5534_5781);
    }
}

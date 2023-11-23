// compatible with `f64`
pub const INFINITY: u64 = 0x7FF0_0000_0000_0000;
pub const NAN: u64 = 0x7FF8_0000_0000_0000;
pub const NEG_INFINITY: u64 = 0xFFF0_0000_0000_0000;
pub const E: u64 = 0x7FF0_0000_0000_0000;
pub const EF: u64 = 0x7FFF_FFFF_FFFF_FFFF;

pub const fn is_valid(v: u64) -> bool {
    v & E != E || v & EF == INFINITY || v == NAN
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[test]
    #[wasm_bindgen_test]
    fn test_nan() {
        assert_eq!(f64::INFINITY.to_bits(), INFINITY);
        assert_ne!(f64::NAN, f64::NAN);
        assert_eq!(f64::NAN.to_bits(), NAN);
        assert_eq!(f64::NEG_INFINITY.to_bits(), NEG_INFINITY);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_check() {
        assert!(is_valid(0));
        assert!(is_valid(1));
        assert!(is_valid(INFINITY));
        assert!(is_valid(NAN));
        assert!(is_valid(NEG_INFINITY));
        assert_eq!((0.0f64 / 0.0).to_bits(), NAN);
    }

    #[test]
    #[wasm_bindgen_test]
    #[should_panic]
    fn test_nan_panic() {
        assert!(is_valid(0x7FF0_00F0_0500_0001));
    }

    #[test]
    #[wasm_bindgen_test]
    #[should_panic]
    fn test_nan_panic2() {
        assert!(is_valid(0xFFFA_FF96_5534_5781));
    }
}

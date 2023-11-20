// compatible with `f64`
pub const INFINITY: u64 = 0x7FF0_0000_0000_0000;
pub const NAN: u64 = 0x7FF8_0000_0000_0000;
pub const NEG_INFINITY: u64 = 0xFFF0_0000_0000_0000;

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
}

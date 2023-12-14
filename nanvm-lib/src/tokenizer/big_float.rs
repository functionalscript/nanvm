use super::big_int::BigInt;

#[derive(Debug, PartialEq, Clone, Eq, Default)]
pub struct BigFloat<const Base: u32> {
    significand: BigInt,
    exp: i64,
}

impl<const Base: u32> BigFloat<Base> {
    pub const ZERO: BigFloat<Base> = BigFloat {
        significand: BigInt::ZERO,
        exp: 0,
    };
}

pub fn dec_to_bin(dec: BigFloat<10>) -> BigFloat<2> {
    if dec.significand.is_zero() {
        return BigFloat::ZERO;
    }

    if dec.exp >= 0 {
        //todo: implement pow for bigint
        todo!()
    }

    todo!()
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::tokenizer::big_int::BigInt;

    use super::{dec_to_bin, BigFloat};

    #[test]
    #[wasm_bindgen_test]
    fn test_zero() {
        let res = dec_to_bin(BigFloat::ZERO);
        assert_eq!(res, BigFloat::ZERO);

        let res = dec_to_bin(BigFloat {
            significand: BigInt::ZERO,
            exp: 10,
        });
        assert_eq!(res, BigFloat::ZERO);

        let res = dec_to_bin(BigFloat {
            significand: BigInt::ZERO,
            exp: -10,
        });
        assert_eq!(res, BigFloat::ZERO);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_integer() {}
}

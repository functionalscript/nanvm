use crate::{common::array::ArrayEx, tokenizer::big_uint::BigUint};

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

    fn increase_significand(mut self, min_significand: BigUint) {
        if self.significand.is_zero() {
            return;
        }

        loop {
            if self.significand.value >= min_significand {
                return;
            }
            //TODO: implement << for BigInt
            //self.significand.value = &self.significand.value << 1;
            self.exp = self.exp - 1;
        }
    }
}

impl BigFloat<10> {
    pub fn to_bin(self) -> BigFloat<2> {
        if self.significand.is_zero() {
            return BigFloat::ZERO;
        }

        if self.exp >= 0 {
            let five = BigUint { value: [5].vec() };
            let new_sign = &self.significand * &five.pow_u64(self.exp as u64).to_big_int();
            todo!()
        }

        todo!()
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::tokenizer::big_int::BigInt;

    use super::BigFloat;

    #[test]
    #[wasm_bindgen_test]
    fn test_zero() {
        let res = BigFloat::ZERO.to_bin();
        assert_eq!(res, BigFloat::ZERO);

        let res = BigFloat {
            significand: BigInt::ZERO,
            exp: 10,
        }
        .to_bin();
        assert_eq!(res, BigFloat::ZERO);

        let res = BigFloat {
            significand: BigInt::ZERO,
            exp: -10,
        }
        .to_bin();
        assert_eq!(res, BigFloat::ZERO);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_integer() {}
}

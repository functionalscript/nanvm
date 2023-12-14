use super::big_int::BigInt;

#[derive(Debug, PartialEq, Clone, Eq, Default)]
pub struct BigFloat {
    significand: BigInt,
    exp: i64,
}

impl BigFloat {
    pub const ZERO: BigFloat = BigFloat {
        significand: BigInt::ZERO,
        exp: 0,
    };
}

pub fn dec_to_bin(dec: BigFloat) -> BigFloat {
    if dec.significand.is_zero() {
        return BigFloat::ZERO;
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
    }
}

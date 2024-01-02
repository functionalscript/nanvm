use crate::{common::cast::Cast, tokenizer::big_uint::BigUint};

use super::big_int::BigInt;

#[derive(Debug, PartialEq, Clone, Eq, Default)]
pub struct BigFloat<const BASE: u32> {
    significand: BigInt,
    exp: i64,
}

impl<const BASE: u32> BigFloat<BASE> {
    pub const ZERO: BigFloat<BASE> = BigFloat {
        significand: BigInt::ZERO,
        exp: 0,
    };

    fn increase_significand(&mut self, min_significand: BigUint) {
        if self.significand.is_zero() {
            return;
        }

        loop {
            if self.significand.value >= min_significand {
                return;
            }
            self.significand.value = &self.significand.value << &BigUint::one();
            self.exp = self.exp - 1;
        }
    }

    fn decrease_significand(&mut self, max_significand: BigUint) -> bool {
        if self.significand.is_zero() {
            return false;
        }

        let mut are_bits_lost = false;
        loop {
            if self.significand.value <= max_significand {
                break;
            }
            let lastBit = self.significand.value.get_last_bit();
            if lastBit == 1 {
                are_bits_lost = true;
            }
            self.significand.value = &self.significand.value >> &BigUint::one();
            self.exp = self.exp + 1;
        }

        are_bits_lost
    }
}

impl BigFloat<10> {
    pub fn to_bin(self, precision: u8) -> BigFloat<2> {
        if self.significand.is_zero() {
            return BigFloat::ZERO;
        }

        if self.exp == 0 {
            return BigFloat {
                significand: self.significand,
                exp: 0,
            };
        }

        let five = BigUint { value: [5].cast() };
        if self.exp > 0 {
            let new_sign = &self.significand * &five.pow_u64(self.exp as u64).to_big_int();
            let result: BigFloat<2> = BigFloat {
                significand: new_sign,
                exp: self.exp,
            };
            return result;
        }

        let p = five.pow_u64(-self.exp as u64);
        let mut bf10 = self.clone();
        let min_significand = &BigUint::one() << &BigUint::from_u64(precision as u64);
        bf10.increase_significand(&p * &min_significand);

        let (q, r) = bf10.significand.div_mod(&p.to_big_int());
        let mut bf2: BigFloat<2> = BigFloat {
            significand: q,
            exp: bf10.exp,
        };
        let max_significand = &min_significand << &BigUint::one();
        let are_bits_lost = bf2.decrease_significand(max_significand);

        let mut last_bit = bf2.significand.value.get_last_bit();
        let abs_value = bf2.significand.value;
        let mut significand = &abs_value >> &BigUint::one();
        let exp = bf2.exp + 1;

        if last_bit == 1 && r.is_zero() && !are_bits_lost {
            last_bit = significand.get_last_bit();
        }

        if last_bit == 1 {
            significand = &significand + &BigUint::one();
        }

        BigFloat {
            significand: BigInt {
                value: significand,
                sign: bf2.significand.sign,
            },
            exp,
        }
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        common::cast::Cast,
        tokenizer::{
            big_int::{BigInt, Sign},
            big_uint::BigUint,
        },
    };

    use super::BigFloat;

    #[test]
    #[wasm_bindgen_test]
    fn test_zero() {
        let res = BigFloat::ZERO.to_bin(64);
        assert_eq!(res, BigFloat::ZERO);

        let res = BigFloat {
            significand: BigInt::ZERO,
            exp: 10,
        }
        .to_bin(64);
        assert_eq!(res, BigFloat::ZERO);

        let res = BigFloat {
            significand: BigInt::ZERO,
            exp: -10,
        }
        .to_bin(64);
        assert_eq!(res, BigFloat::ZERO);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_integer() {
        let a = BigFloat {
            significand: BigInt::from_i64(100),
            exp: 0,
        };
        let res = a.to_bin(64);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(100),
                exp: 0,
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(1),
            exp: 1,
        };
        let res = a.to_bin(64);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(5),
                exp: 1,
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(100),
            exp: 2,
        };
        let res = a.to_bin(64);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(2500),
                exp: 2,
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_float() {
        let a = BigFloat {
            significand: BigInt::from_i64(100),
            exp: -1,
        };
        let res = a.to_bin(5);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(20),
                exp: -1,
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(100),
            exp: -1,
        };
        let res = a.to_bin(64);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt {
                    sign: Sign::Positive,
                    value: BigUint {
                        value: [(1 << 63) + (1 << 61)].cast()
                    }
                },
                exp: -60,
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_round() {
        let a = BigFloat {
            significand: BigInt::from_i64(0b1000_0001),
            exp: -1,
        };
        let res = a.to_bin(5);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(0b11010),
                exp: -1,
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(0b1000_0001),
            exp: -1,
        };
        let res = a.to_bin(4);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(0b1101),
                exp: 0,
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(0b1000_0001),
            exp: -1,
        };
        let res = a.to_bin(3);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(0b110),
                exp: 1,
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_round_half() {
        let a = BigFloat {
            significand: BigInt::from_i64(0b101_1010),
            exp: -1,
        };
        let res = a.to_bin(3);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(0b100),
                exp: 1,
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(0b101_1011),
            exp: -1,
        };
        let res = a.to_bin(3);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(0b101),
                exp: 1,
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(0b110_1101),
            exp: -1,
        };
        let res = a.to_bin(3);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(0b101),
                exp: 1,
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(0b110_1110),
            exp: -1,
        };
        let res = a.to_bin(3);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(0b110),
                exp: 1,
            }
        );

        // let a = BigFloat {
        //     significand: BigInt::from_i64(0b1001_0110),
        //     exp: -1,
        // };
        // let res = a.to_bin(3);
        // assert_eq!(
        //     res,
        //     BigFloat {
        //         significand: BigInt::from_i64(0b100),
        //         exp: 2,
        //     }
        // );
    }
}

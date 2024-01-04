use crate::{
    common::cast::Cast,
    tokenizer::{big_int::Sign, big_uint::BigUint},
};

use super::big_int::BigInt;

#[derive(Debug, PartialEq, Clone, Eq, Default)]
pub struct BigFloat<const BASE: u32> {
    pub significand: BigInt,
    pub exp: i64,
}

impl<const BASE: u32> BigFloat<BASE> {
    pub const ZERO: BigFloat<BASE> = BigFloat {
        significand: BigInt::ZERO,
        exp: 0,
    };

    fn increase_significand(&mut self, min_significand: &BigUint) {
        if self.significand.is_zero() {
            return;
        }

        loop {
            if self.significand.value >= *min_significand {
                return;
            }
            self.significand.value = &self.significand.value << &BigUint::one();
            self.exp = self.exp - 1;
        }
    }

    fn decrease_significand(&mut self, max_significand: &BigUint) -> bool {
        if self.significand.is_zero() {
            return false;
        }

        let mut are_bits_lost = false;
        loop {
            if self.significand.value < *max_significand {
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
        bf10.increase_significand(&(&p * &min_significand));

        let (q, r) = bf10.significand.div_mod(&p.to_big_int());
        let mut bf2: BigFloat<2> = BigFloat {
            significand: q,
            exp: bf10.exp,
        };
        let max_significand = &min_significand << &BigUint::one();
        let are_bits_lost = bf2.decrease_significand(&max_significand);

        let mut last_bit = bf2.significand.value.get_last_bit();
        let abs_value = bf2.significand.value;
        let mut significand = &abs_value >> &BigUint::one();
        let mut exp = bf2.exp + 1;

        if last_bit == 1 && r.is_zero() && !are_bits_lost {
            last_bit = significand.get_last_bit();
        }

        if last_bit == 1 {
            significand = &significand + &BigUint::one();
            if significand.eq(&min_significand) {
                significand = &significand >> &BigUint::one();
                exp = exp + 1;
            }
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

impl BigFloat<2> {
    const PRECISION: u64 = 52;
    const DEFAULT_EXP: u64 = 1023;
    const FRAC_MASK: u64 = (1 << Self::PRECISION) - 1;

    fn to_f64(self) -> f64 {
        f64::from_bits(self.get_f64_bits())
    }

    fn get_f64_bits(self) -> u64 {
        let mut bits: u64 = 0;
        if self.significand.sign == Sign::Negative {
            bits = bits | 1 << 63;
        }

        if self.significand.is_zero() {
            return bits;
        }

        let mut value = self.clone();
        let min_significand = &BigUint::one() << &BigUint::from_u64(Self::PRECISION);
        value.increase_significand(&min_significand);
        let max_significand = &min_significand << &BigUint::one();
        value.decrease_significand(&max_significand);

        let f64_exp = value.exp + Self::PRECISION as i64;
        match f64_exp {
            -1022..=1023 => {
                let exp_bits = (f64_exp + 1023) as u64;
                bits = bits | exp_bits << 52;
                let frac_bits = value.significand.value.value[0] & Self::FRAC_MASK;
                bits = bits | frac_bits;
                bits
            }
            _ => todo!(),
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
        let res = a.to_bin(7);
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

        let a = BigFloat {
            significand: BigInt {
                sign: Sign::Positive,
                value: BigUint {
                    value: [0, 1].cast(),
                },
            },
            exp: 0,
        };
        let res = a.to_bin(53);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt {
                    sign: Sign::Positive,
                    value: BigUint {
                        value: [1 << 52].cast()
                    }
                },
                exp: 12,
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

        let a = BigFloat {
            significand: BigInt::from_i64(0b1001_0110),
            exp: -1,
        };
        let res = a.to_bin(3);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(0b100),
                exp: 2,
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_zero_to_f64() {
        let a = BigFloat {
            significand: BigInt::ZERO,
            exp: 100,
        };
        let res = a.to_f64();
        assert_eq!(res, 0.0);
        assert!(res.is_sign_positive());

        let a = BigFloat {
            significand: BigInt {
                sign: Sign::Negative,
                value: BigUint::ZERO,
            },
            exp: 100,
        };
        let res = a.to_f64();
        assert_eq!(res, 0.0);
        assert!(res.is_sign_negative());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_normal_to_f64() {
        let a = BigFloat {
            significand: BigInt::from_u64(1),
            exp: 0,
        };
        let res = a.to_f64();
        assert_eq!(res, 1.0);

        let a = BigFloat {
            significand: BigInt::from_i64(-3),
            exp: -1,
        };
        let res = a.to_f64();
        assert_eq!(res, -1.5);

        let a = BigFloat {
            significand: BigInt::from_i64(1),
            exp: -1022,
        };
        let res = a.to_f64();
        assert_eq!(res, 2.0f64.powf(-1022.0));

        let a = BigFloat {
            significand: BigInt::from_i64(1),
            exp: 1023,
        };
        let res = a.to_f64();
        assert_eq!(res, 2.0f64.powf(1023.0));
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_infinity_to_f64() {
    }
}

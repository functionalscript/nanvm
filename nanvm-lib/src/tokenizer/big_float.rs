use crate::{
    common::cast::Cast,
    tokenizer::{big_int::Sign, big_uint::BigUint},
};

use super::big_int::BigInt;

#[derive(Debug, PartialEq, Clone, Eq, Default)]
pub struct BigFloat<const BASE: u32> {
    pub significand: BigInt,
    pub exp: i64,
    pub non_zero_reminder: bool,
}

impl<const BASE: u32> BigFloat<BASE> {
    pub const ZERO: BigFloat<BASE> = BigFloat {
        significand: BigInt::ZERO,
        exp: 0,
        non_zero_reminder: false,
    };

    fn increase_significand(&mut self, precision: u64) {
        if self.significand.is_zero() {
            return;
        }

        let min_significand = &BigUint::one() << &BigUint::from_u64(precision as u64);
        self.increase_significand_to(&min_significand);
    }

    fn increase_significand_to(&mut self, min_significand: &BigUint) {
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

    fn decrease_significand(&mut self, precision: u64) {
        if self.significand.is_zero() {
            return;
        }

        let max_significand = &BigUint::one() << &BigUint::from_u64(precision as u64);
        loop {
            if self.significand.value < max_significand {
                break;
            }
            let lastBit = self.significand.value.get_last_bit();
            if lastBit == 1 {
                self.non_zero_reminder = true;
            }
            self.significand.value = &self.significand.value >> &BigUint::one();
            self.exp = self.exp + 1;
        }
    }
}

impl BigFloat<10> {
    pub fn to_bin(self, precision: u8) -> BigFloat<2> {
        if self.significand.is_zero() {
            return BigFloat::ZERO;
        }

        if self.exp == 0 {
            let mut result: BigFloat<2> = BigFloat {
                significand: self.significand,
                exp: self.exp,
                non_zero_reminder: self.non_zero_reminder,
            };
            result.increase_significand(precision as u64);
            result.decrease_significand(precision as u64);
            return result;
        }

        let five = BigUint { value: [5].cast() };
        if self.exp > 0 {
            let new_sign = &self.significand * &five.pow_u64(self.exp as u64).to_big_int();
            let mut result: BigFloat<2> = BigFloat {
                significand: new_sign,
                exp: self.exp,
                non_zero_reminder: self.non_zero_reminder,
            };
            result.increase_significand(precision as u64);
            result.decrease_significand(precision as u64);
            return result;
        }

        let p = five.pow_u64(-self.exp as u64);
        let mut bf10 = self.clone();
        let min_significand = &BigUint::one() << &BigUint::from_u64(precision as u64);
        bf10.increase_significand_to(&(&p * &min_significand));

        let (q, r) = bf10.significand.div_mod(&p.to_big_int());
        let mut result: BigFloat<2> = BigFloat {
            significand: q,
            exp: bf10.exp,
            non_zero_reminder: self.non_zero_reminder || !r.is_zero(),
        };
        result.decrease_significand(precision as u64);
        result
    }
}

impl BigFloat<2> {
    fn to_f64(self) -> f64 {
        f64::from_bits(self.get_f64_bits())
    }

    fn get_f64_bits(self) -> u64 {
        const PRECISION: u64 = 52;
        const MAX_FRAC: u64 = 1 << (PRECISION + 1);
        const FRAC_MASK: u64 = (1 << PRECISION) - 1;
        const INF_BITS: u64 = 2047 << 52;

        let mut bits: u64 = 0;
        if self.significand.sign == Sign::Negative {
            bits = bits | 1 << 63;
        }

        if self.significand.is_zero() {
            return bits;
        }

        let mut value = self.clone();
        value.increase_significand(PRECISION + 1);
        value.decrease_significand(PRECISION + 2);

        let mut f64_exp = value.exp + PRECISION as i64 + 1;
        match f64_exp {
            -1022..=1023 => {
                let mut last_bit = value.significand.value.get_last_bit();
                let mut frac = value.significand.value.value[0] >> 1;

                if last_bit == 1 && !value.non_zero_reminder {
                    last_bit = frac & 1;
                }
                if last_bit == 1 {
                    frac = frac + 1;
                    if frac == MAX_FRAC {
                        frac = frac >> 1;
                        f64_exp = f64_exp + 1;
                    }
                }

                let exp_bits = (f64_exp + 1023) as u64;
                bits = bits | exp_bits << 52;
                let frac_bits = frac & FRAC_MASK;
                bits = bits | frac_bits;
                bits
            }
            -1074..=-1023 => {
                let exp_dif = -1021 - f64_exp;
                let frac_bits = value.significand.value.value[0] >> exp_dif;
                bits = bits | frac_bits;
                bits
            }
            exp if exp > 1023 => {
                bits = bits | INF_BITS;
                bits
            }
            _ => bits,
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
            non_zero_reminder: false,
        }
        .to_bin(64);
        assert_eq!(res, BigFloat::ZERO);

        let res = BigFloat {
            significand: BigInt::ZERO,
            exp: -10,
            non_zero_reminder: false,
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
            non_zero_reminder: false,
        };
        let res = a.to_bin(7);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(100),
                exp: 0,
                non_zero_reminder: false
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(1),
            exp: 1,
            non_zero_reminder: false,
        };
        let res = a.to_bin(64);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_u64(10 << 60),
                exp: -60,
                non_zero_reminder: false
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(100),
            exp: 2,
            non_zero_reminder: false,
        };
        let res = a.to_bin(64);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_u64(10000 << 50),
                exp: -50,
                non_zero_reminder: false
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(128),
            exp: 0,
            non_zero_reminder: false,
        };
        let res = a.to_bin(9);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(256),
                exp: -1,
                non_zero_reminder: false
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_integer_round() {
        let a = BigFloat {
            significand: BigInt::from_i64(128),
            exp: 0,
            non_zero_reminder: false,
        };
        let res = a.to_bin(4);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(8),
                exp: 4,
                non_zero_reminder: false
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(129),
            exp: 0,
            non_zero_reminder: false,
        };
        let res = a.to_bin(4);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(8),
                exp: 4,
                non_zero_reminder: true
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_float() {
        let a = BigFloat {
            significand: BigInt::from_i64(100),
            exp: -1,
            non_zero_reminder: false,
        };
        let res = a.to_bin(5);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(20),
                exp: -1,
                non_zero_reminder: false
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(100),
            exp: -1,
            non_zero_reminder: false,
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
                non_zero_reminder: false
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
            non_zero_reminder: false,
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
                non_zero_reminder: false
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_round() {
        let a = BigFloat {
            significand: BigInt::from_i64(0b1000_0001),
            exp: -1,
            non_zero_reminder: false,
        };
        let res = a.to_bin(5);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(0b11001),
                exp: -1,
                non_zero_reminder: true
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(0b1000_0001),
            exp: -1,
            non_zero_reminder: false,
        };
        let res = a.to_bin(4);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(0b1100),
                exp: 0,
                non_zero_reminder: true
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(0b1000_0001),
            exp: -1,
            non_zero_reminder: false,
        };
        let res = a.to_bin(3);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(0b110),
                exp: 1,
                non_zero_reminder: true
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_round_half() {
        let a = BigFloat {
            significand: BigInt::from_i64(0b101_1010),
            exp: -1,
            non_zero_reminder: false,
        };
        let res = a.to_bin(3);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(0b100),
                exp: 1,
                non_zero_reminder: true
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(0b101_1011),
            exp: -1,
            non_zero_reminder: false,
        };
        let res = a.to_bin(3);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(0b100),
                exp: 1,
                non_zero_reminder: true
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(0b110_1101),
            exp: -1,
            non_zero_reminder: false,
        };
        let res = a.to_bin(3);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(0b101),
                exp: 1,
                non_zero_reminder: true
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(0b110_1110),
            exp: -1,
            non_zero_reminder: false,
        };
        let res = a.to_bin(3);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(0b101),
                exp: 1,
                non_zero_reminder: true
            }
        );

        let a = BigFloat {
            significand: BigInt::from_i64(0b1001_0110),
            exp: -1,
            non_zero_reminder: false,
        };
        let res = a.to_bin(3);
        assert_eq!(
            res,
            BigFloat {
                significand: BigInt::from_i64(0b111),
                exp: 1,
                non_zero_reminder: true
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_zero_to_f64() {
        let a = BigFloat {
            significand: BigInt::ZERO,
            exp: 100,
            non_zero_reminder: false,
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
            non_zero_reminder: false,
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
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert_eq!(res, 1.0);

        let a = BigFloat {
            significand: BigInt::from_i64(-3),
            exp: -1,
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert_eq!(res, -1.5);

        let a = BigFloat {
            significand: BigInt::from_i64(1),
            exp: -1022,
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert_eq!(res, 2.0f64.powf(-1022.0));

        let a = BigFloat {
            significand: BigInt::from_i64(1),
            exp: 1023,
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert_eq!(res, 2.0f64.powf(1023.0));

        let a = BigFloat {
            significand: BigInt::from_u64((1 << 52) - 1),
            exp: 0,
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert_eq!(res, 4503599627370495f64);

        let a = BigFloat {
            significand: BigInt::from_u64((1 << 53) - 1),
            exp: 0,
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert_eq!(res, 9007199254740991f64);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_normal_to_f64_round() {
        let a = BigFloat {
            significand: BigInt::from_u64((1 << 54) - 1),
            exp: 0,
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert_eq!(res, 18014398509481983f64);

        let a = BigFloat {
            significand: BigInt::from_u64((1 << 54) - 2),
            exp: 0,
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert_eq!(res, 18014398509481982f64);

        let a = BigFloat {
            significand: BigInt::from_u64((1 << 54) - 3),
            exp: 0,
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert_eq!(res, 18014398509481980f64);

        let a = BigFloat {
            significand: BigInt::from_u64((1 << 54) - 1),
            exp: 969,
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert!(res.is_normal());

        let a = BigFloat {
            significand: BigInt::from_u64((1 << 54) - 1),
            exp: 970,
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert!(res.is_infinite());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_infinity_to_f64() {
        let a = BigFloat {
            significand: BigInt::from_i64(1),
            exp: 1024,
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert!(res.is_infinite());
        assert!(res.is_sign_positive());

        let a = BigFloat {
            significand: BigInt::from_i64(-1),
            exp: 1024,
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert!(res.is_infinite());
        assert!(res.is_sign_negative());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_subnormal_to_f64() {
        let a = BigFloat {
            significand: BigInt::from_u64(1),
            exp: -1023,
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert_eq!(res, 2.0f64.powf(-1023.0));
        assert!(res.is_subnormal());

        let a = BigFloat {
            significand: BigInt::from_i64(-1),
            exp: -1023,
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert_eq!(res, -2.0f64.powf(-1023.0));
        assert!(res.is_subnormal());

        let a = BigFloat {
            significand: BigInt::from_u64(1),
            exp: -1074,
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert_eq!(res.to_bits(), 1);
        assert_eq!(res, 2.0f64.powf(-1074.0));
        assert!(res.is_subnormal());

        let a = BigFloat {
            significand: BigInt::from_u64(1),
            exp: -1075,
            non_zero_reminder: false,
        };
        let res = a.to_f64();
        assert_eq!(res, 0.0);
    }
}

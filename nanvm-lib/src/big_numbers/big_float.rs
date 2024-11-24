use std::{cmp::Ordering, ops::Deref};

use crate::{
    common::cast::Cast,
    js::js_bigint::{
        div_mod, from_u64, is_zero, mul, pow, pow_u64, shl, shl_on_u64, shr_on_u64, zero, JsBigint,
        JsBigintMutRef, Sign,
    },
    mem::manager::Manager,
};

#[derive(Debug)]
pub struct BigFloat<const BASE: u32, M: Manager> {
    pub manager: M,
    pub significand: JsBigintMutRef<M::Dealloc>,
    pub exp: i64,
    pub non_zero_reminder: bool,
}

pub fn float_zero<const BASE: u32, M: Manager>(manager: M) -> BigFloat<BASE, M> {
    BigFloat {
        manager,
        significand: zero(manager),
        exp: 0,
        non_zero_reminder: false,
    }
}

impl<const BASE: u32, M: Manager> BigFloat<BASE, M> {
    fn increase_significand(&mut self, precision: u64) {
        if is_zero(self.significand.deref()) {
            return;
        }

        let min_significand = shl(
            self.manager,
            from_u64(self.manager, Sign::Positive, 1).deref(),
            from_u64(self.manager, Sign::Positive, precision).deref(),
        );
        self.increase_significand_to(min_significand.deref());
    }

    fn increase_significand_to(&mut self, min_significand: &JsBigint) {
        if is_zero(self.significand.deref()) {
            return;
        }

        loop {
            match self.significand.deref().cmp(min_significand) {
                Ordering::Greater | Ordering::Equal => return,
                _ => {}
            }
            self.significand = shl_on_u64(self.manager, self.significand.deref(), 1);
            self.exp -= 1;
        }
    }

    fn decrease_significand(&mut self, precision: u64) {
        if is_zero(self.significand.deref()) {
            return;
        }

        let max_significand = shl(
            self.manager,
            from_u64(self.manager, Sign::Positive, 1).deref(),
            from_u64(self.manager, Sign::Positive, precision).deref(),
        );
        loop {
            if self.significand.deref().cmp(max_significand.deref()) == Ordering::Less {
                break;
            }
            let last_bit = self.significand.get_last_bit();
            if last_bit == 1 {
                self.non_zero_reminder = true;
            }
            self.significand = shr_on_u64(self.manager, self.significand.deref(), 1);
            self.exp += 1;
        }
    }
}

impl<M: Manager> BigFloat<10, M> {
    pub fn to_bin(self, precision: u8) -> BigFloat<2, M> {
        if is_zero(self.significand.deref()) {
            return float_zero(self.manager);
        }

        if self.exp == 0 {
            let mut result: BigFloat<2, M> = BigFloat {
                manager: self.manager,
                significand: self.significand,
                exp: self.exp,
                non_zero_reminder: self.non_zero_reminder,
            };
            result.increase_significand(precision as u64);
            result.decrease_significand(precision as u64);
            return result;
        }

        let five = from_u64(self.manager, Sign::Positive, 5);
        if self.exp > 0 {
            let new_sign = mul(
                self.manager,
                self.significand.deref(),
                pow_u64(self.manager, five.deref(), self.exp as u64).deref(),
            );
            let mut result: BigFloat<2, M> = BigFloat {
                manager: self.manager,
                significand: new_sign,
                exp: self.exp,
                non_zero_reminder: self.non_zero_reminder,
            };
            result.increase_significand(precision as u64);
            result.decrease_significand(precision as u64);
            return result;
        }

        let p = pow_u64(self.manager, five.deref(), -self.exp as u64);
        let mut bf10: BigFloat<10, M> = BigFloat {
            manager: self.manager,
            significand: self.significand,
            exp: self.exp,
            non_zero_reminder: self.non_zero_reminder,
        };
        let min_significand = shl(
            self.manager,
            from_u64(self.manager, Sign::Positive, 1).deref(),
            from_u64(self.manager, Sign::Positive, precision as u64).deref(),
        );
        bf10.increase_significand_to(
            (mul(self.manager, p.deref(), min_significand.deref())).deref(),
        );

        let (q, r) = div_mod(self.manager, bf10.significand.deref(), p.deref());
        let mut result: BigFloat<2, M> = BigFloat {
            manager: self.manager,
            significand: q,
            exp: bf10.exp,
            non_zero_reminder: self.non_zero_reminder || !is_zero(r.deref()),
        };
        result.decrease_significand(precision as u64);
        result
    }
}

impl<M: Manager> BigFloat<2, M> {
    fn get_frac_round(self) -> u64 {
        let mut last_bit = self.significand.get_last_bit();
        let mut frac = self.significand.items()[0] >> 1;

        if last_bit == 1 && !self.non_zero_reminder {
            last_bit = frac & 1;
        }

        if last_bit == 1 {
            frac += 1;
        }

        frac
    }

    pub fn to_f64(self) -> f64 {
        f64::from_bits(self.get_f64_bits())
    }

    fn get_f64_bits(self) -> u64 {
        const PRECISION: u64 = 52;
        const MAX_FRAC: u64 = 1 << (PRECISION + 1);
        const FRAC_MASK: u64 = (1 << PRECISION) - 1;
        const INF_BITS: u64 = 2047 << 52;

        let mut bits: u64 = 0;
        if self.significand.header_len() < 0 {
            bits |= 1 << 63;
        }

        if is_zero(self.significand.deref()) {
            return bits;
        }

        let mut value = BigFloat {
            manager: self.manager,
            significand: self.significand,
            exp: self.exp,
            non_zero_reminder: self.non_zero_reminder,
        };
        value.increase_significand(PRECISION + 1);
        value.decrease_significand(PRECISION + 2);

        let mut f64_exp = value.exp + PRECISION as i64 + 1;
        match f64_exp {
            -1022..=1023 => {
                let mut frac = value.get_frac_round();
                if frac == MAX_FRAC {
                    frac >>= 1;
                    f64_exp += 1;
                    //if f64_exp equals 1024, then exp_bits will be all ones and frac_bits will be all zeros
                    //it is an infinity by the f64 standard
                }

                let exp_bits = (f64_exp + 1023) as u64;
                bits |= exp_bits << 52;
                let frac_bits = frac & FRAC_MASK;
                bits |= frac_bits;
                bits
            }
            -1074..=-1023 => {
                let subnormal_precision = (f64_exp + 1076) as u64;
                value.decrease_significand(subnormal_precision);
                let frac = value.get_frac_round();
                //if frac equals 1 << 53, then exp_bits will be 1 and frac_bits will be all zeros
                //it is a normal number by the f64 standard and it is the correct number
                bits |= frac;
                bits
            }
            exp if exp > 1023 => {
                bits |= INF_BITS;
                bits
            }
            _ => bits,
        }
    }
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        big_numbers::big_float::float_zero,
        common::cast::Cast,
        js::{
            any::Any,
            js_bigint::{self, from_u64, new_bigint, zero, JsBigintRef, Sign},
            type_::Type,
        },
        mem::global::{Global, GLOBAL},
    };

    use super::BigFloat;

    #[test]
    #[wasm_bindgen_test]
    fn test_zero() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let res = float_zero(GLOBAL).to_bin(64);
        let any = A::move_from(res.significand.to_ref());
        assert_eq!(any.get_type(), Type::Bigint);
        {
            let o = any.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert!(o.items().is_empty());
        }
        assert_eq!(res.exp, 0);
        assert_eq!(res.non_zero_reminder, false);

        let res = BigFloat {
            manager: GLOBAL,
            significand: zero(GLOBAL),
            exp: 10,
            non_zero_reminder: false,
        }
        .to_bin(64);
        let any = A::move_from(res.significand.to_ref());
        assert_eq!(any.get_type(), Type::Bigint);
        {
            let o = any.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert!(o.items().is_empty());
        }
        assert_eq!(res.exp, 0);
        assert_eq!(res.non_zero_reminder, false);

        let res = BigFloat {
            manager: GLOBAL,
            significand: zero(GLOBAL),
            exp: -10,
            non_zero_reminder: false,
        }
        .to_bin(64);
        let any = A::move_from(res.significand.to_ref());
        assert_eq!(any.get_type(), Type::Bigint);
        {
            let o = any.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert!(o.items().is_empty());
        }
        assert_eq!(res.exp, 0);
        assert_eq!(res.non_zero_reminder, false);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_integer() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let res = BigFloat {
            manager: GLOBAL,
            significand: from_u64(GLOBAL, Sign::Positive, 100),
            exp: 0,
            non_zero_reminder: false,
        }
        .to_bin(7);
        let any = A::move_from(res.significand.to_ref());
        assert_eq!(any.get_type(), Type::Bigint);
        {
            let o = any.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[100]);
        }
        assert_eq!(res.exp, 0);
        assert_eq!(res.non_zero_reminder, false);

        let res = BigFloat {
            manager: GLOBAL,
            significand: from_u64(GLOBAL, Sign::Positive, 1),
            exp: 1,
            non_zero_reminder: false,
        }
        .to_bin(64);
        let any = A::move_from(res.significand.to_ref());
        assert_eq!(any.get_type(), Type::Bigint);
        {
            let o = any.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[10 << 60]);
        }
        assert_eq!(res.exp, -60);
        assert_eq!(res.non_zero_reminder, false);

        let res = BigFloat {
            manager: GLOBAL,
            significand: from_u64(GLOBAL, Sign::Positive, 100),
            exp: 2,
            non_zero_reminder: false,
        }
        .to_bin(64);
        let any = A::move_from(res.significand.to_ref());
        assert_eq!(any.get_type(), Type::Bigint);
        {
            let o = any.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[10000 << 50]);
        }
        assert_eq!(res.exp, -50);
        assert_eq!(res.non_zero_reminder, false);

        let res = BigFloat {
            manager: GLOBAL,
            significand: from_u64(GLOBAL, Sign::Positive, 128),
            exp: 0,
            non_zero_reminder: false,
        }
        .to_bin(9);
        let any = A::move_from(res.significand.to_ref());
        assert_eq!(any.get_type(), Type::Bigint);
        {
            let o = any.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[256]);
        }
        assert_eq!(res.exp, -1);
        assert_eq!(res.non_zero_reminder, false);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_integer_rounding() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let res = BigFloat {
            manager: GLOBAL,
            significand: from_u64(GLOBAL, Sign::Positive, 128),
            exp: 0,
            non_zero_reminder: false,
        }
        .to_bin(4);
        let any = A::move_from(res.significand.to_ref());
        assert_eq!(any.get_type(), Type::Bigint);
        {
            let o = any.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[8]);
        }
        assert_eq!(res.exp, 4);
        assert_eq!(res.non_zero_reminder, false);

        let res = BigFloat {
            manager: GLOBAL,
            significand: from_u64(GLOBAL, Sign::Positive, 129),
            exp: 0,
            non_zero_reminder: false,
        }
        .to_bin(4);
        let any = A::move_from(res.significand.to_ref());
        assert_eq!(any.get_type(), Type::Bigint);
        {
            let o = any.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[8]);
        }
        assert_eq!(res.exp, 4);
        assert_eq!(res.non_zero_reminder, true);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_float() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let res = BigFloat {
            manager: GLOBAL,
            significand: from_u64(GLOBAL, Sign::Positive, 100),
            exp: -1,
            non_zero_reminder: false,
        }
        .to_bin(5);
        let any = A::move_from(res.significand.to_ref());
        assert_eq!(any.get_type(), Type::Bigint);
        {
            let o = any.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[20]);
        }
        assert_eq!(res.exp, -1);
        assert_eq!(res.non_zero_reminder, false);

        let res = BigFloat {
            manager: GLOBAL,
            significand: from_u64(GLOBAL, Sign::Positive, 100),
            exp: -1,
            non_zero_reminder: false,
        }
        .to_bin(64);
        let any = A::move_from(res.significand.to_ref());
        assert_eq!(any.get_type(), Type::Bigint);
        {
            let o = any.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[(1 << 63) + (1 << 61)]);
        }
        assert_eq!(res.exp, -60);
        assert_eq!(res.non_zero_reminder, false);

        let res = BigFloat {
            manager: GLOBAL,
            significand: new_bigint(GLOBAL, Sign::Positive, [0, 1]),
            exp: 0,
            non_zero_reminder: false,
        }
        .to_bin(53);
        let any = A::move_from(res.significand.to_ref());
        assert_eq!(any.get_type(), Type::Bigint);
        {
            let o = any.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1 << 52]);
        }
        assert_eq!(res.exp, 12);
        assert_eq!(res.non_zero_reminder, false);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_rounding() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let res = BigFloat {
            manager: GLOBAL,
            significand: from_u64(GLOBAL, Sign::Positive, 0b1000_0001),
            exp: -1,
            non_zero_reminder: false,
        }
        .to_bin(5);
        let any = A::move_from(res.significand.to_ref());
        assert_eq!(any.get_type(), Type::Bigint);
        {
            let o = any.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[0b11001]);
        }
        assert_eq!(res.exp, -1);
        assert_eq!(res.non_zero_reminder, true);

        let res = BigFloat {
            manager: GLOBAL,
            significand: from_u64(GLOBAL, Sign::Positive, 0b1000_0001),
            exp: -1,
            non_zero_reminder: false,
        }
        .to_bin(4);
        let any = A::move_from(res.significand.to_ref());
        assert_eq!(any.get_type(), Type::Bigint);
        {
            let o = any.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[0b1100]);
        }
        assert_eq!(res.exp, 0);
        assert_eq!(res.non_zero_reminder, true);

        let res = BigFloat {
            manager: GLOBAL,
            significand: from_u64(GLOBAL, Sign::Positive, 0b1000_0001),
            exp: -1,
            non_zero_reminder: false,
        }
        .to_bin(3);
        let any = A::move_from(res.significand.to_ref());
        assert_eq!(any.get_type(), Type::Bigint);
        {
            let o = any.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[0b110]);
        }
        assert_eq!(res.exp, 1);
        assert_eq!(res.non_zero_reminder, true);
    }

    // #[test]
    // #[wasm_bindgen_test]
    // fn test_rounding_half() {
    //     let a = BigFloat {
    //         significand: BigInt::from_i64(0b101_1010),
    //         exp: -1,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_bin(3);
    //     assert_eq!(
    //         res,
    //         BigFloat {
    //             significand: BigInt::from_i64(0b100),
    //             exp: 1,
    //             non_zero_reminder: true
    //         }
    //     );

    //     let a = BigFloat {
    //         significand: BigInt::from_i64(0b101_1011),
    //         exp: -1,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_bin(3);
    //     assert_eq!(
    //         res,
    //         BigFloat {
    //             significand: BigInt::from_i64(0b100),
    //             exp: 1,
    //             non_zero_reminder: true
    //         }
    //     );

    //     let a = BigFloat {
    //         significand: BigInt::from_i64(0b110_1101),
    //         exp: -1,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_bin(3);
    //     assert_eq!(
    //         res,
    //         BigFloat {
    //             significand: BigInt::from_i64(0b101),
    //             exp: 1,
    //             non_zero_reminder: true
    //         }
    //     );

    //     let a = BigFloat {
    //         significand: BigInt::from_i64(0b110_1110),
    //         exp: -1,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_bin(3);
    //     assert_eq!(
    //         res,
    //         BigFloat {
    //             significand: BigInt::from_i64(0b101),
    //             exp: 1,
    //             non_zero_reminder: true
    //         }
    //     );

    //     let a = BigFloat {
    //         significand: BigInt::from_i64(0b1001_0110),
    //         exp: -1,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_bin(3);
    //     assert_eq!(
    //         res,
    //         BigFloat {
    //             significand: BigInt::from_i64(0b111),
    //             exp: 1,
    //             non_zero_reminder: true
    //         }
    //     );
    // }

    // #[test]
    // #[wasm_bindgen_test]
    // fn test_zero_to_f64() {
    //     let a = BigFloat {
    //         significand: BigInt::ZERO,
    //         exp: 100,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, 0.0);
    //     assert!(res.is_sign_positive());

    //     let a = BigFloat {
    //         significand: BigInt {
    //             sign: Sign::Negative,
    //             value: BigUint::ZERO,
    //         },
    //         exp: 100,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, 0.0);
    //     assert!(res.is_sign_negative());
    // }

    // #[test]
    // #[wasm_bindgen_test]
    // fn test_normal_to_f64() {
    //     let a = BigFloat {
    //         significand: BigInt::from_u64(1),
    //         exp: 0,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, 1.0);

    //     let a = BigFloat {
    //         significand: BigInt::from_i64(-3),
    //         exp: -1,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, -1.5);

    //     let a = BigFloat {
    //         significand: BigInt::from_i64(1),
    //         exp: -1022,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, 2.0f64.powf(-1022.0));
    //     assert!(res.is_normal());

    //     let a = BigFloat {
    //         significand: BigInt::from_u64(1 << 59),
    //         exp: -1022 - 59,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, 2.0f64.powf(-1022.0));
    //     assert!(res.is_normal());

    //     let a = BigFloat {
    //         significand: BigInt::from_u64((1 << 60) - 1),
    //         exp: -1022 - 60,
    //         non_zero_reminder: true,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, 2.0f64.powf(-1022.0));
    //     assert!(res.is_normal());

    //     let a = BigFloat {
    //         significand: BigInt::from_i64(1),
    //         exp: 1023,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, 2.0f64.powf(1023.0));

    //     let a = BigFloat {
    //         significand: BigInt::from_u64((1 << 52) - 1),
    //         exp: 0,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, 4503599627370495f64);

    //     let a = BigFloat {
    //         significand: BigInt::from_u64((1 << 53) - 1),
    //         exp: 0,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, 9007199254740991f64);
    // }

    // #[test]
    // #[wasm_bindgen_test]
    // fn test_normal_to_f64_rounding() {
    //     let a = BigFloat {
    //         significand: BigInt::from_u64((1 << 54) - 1), //111111111111111111111111111111111111111111111111111111
    //         exp: 0,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, 18014398509481984f64);

    //     let a = BigFloat {
    //         significand: BigInt::from_u64((1 << 54) - 2), //111111111111111111111111111111111111111111111111111110
    //         exp: 0,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, 18014398509481982f64);

    //     let a = BigFloat {
    //         significand: BigInt::from_u64((1 << 54) - 3), //111111111111111111111111111111111111111111111111111101
    //         exp: 0,
    //         non_zero_reminder: true,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, 18014398509481982f64);

    //     let a = BigFloat {
    //         significand: BigInt::from_u64((1 << 54) - 3), //111111111111111111111111111111111111111111111111111101
    //         exp: 0,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, 18014398509481980f64);

    //     let a = BigFloat {
    //         significand: BigInt::from_u64((1 << 54) - 1),
    //         exp: 969,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert!(res.is_normal());

    //     let a = BigFloat {
    //         significand: BigInt::from_u64((1 << 54) - 1),
    //         exp: 970,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert!(res.is_infinite());
    // }

    // #[test]
    // #[wasm_bindgen_test]
    // fn test_infinity_to_f64() {
    //     let a = BigFloat {
    //         significand: BigInt::from_i64(1),
    //         exp: 1024,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert!(res.is_infinite());
    //     assert!(res.is_sign_positive());

    //     let a = BigFloat {
    //         significand: BigInt::from_i64(-1),
    //         exp: 1024,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert!(res.is_infinite());
    //     assert!(res.is_sign_negative());
    // }

    // #[test]
    // #[wasm_bindgen_test]
    // fn test_subnormal_to_f64() {
    //     let a = BigFloat {
    //         significand: BigInt::from_u64(1),
    //         exp: -1023,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, 2.0f64.powf(-1023.0));
    //     assert!(res.is_subnormal());

    //     let a = BigFloat {
    //         significand: BigInt::from_i64(-1),
    //         exp: -1023,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, -(2.0f64.powf(-1023.0)));
    //     assert!(res.is_subnormal());

    //     let a = BigFloat {
    //         significand: BigInt::from_u64(1),
    //         exp: -1074,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res.to_bits(), 1);
    //     assert_eq!(res, 2.0f64.powf(-1074.0));
    //     assert!(res.is_subnormal());

    //     let a = BigFloat {
    //         significand: BigInt::from_u64(1),
    //         exp: -1075,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res, 0.0);
    // }

    // #[test]
    // #[wasm_bindgen_test]
    // fn test_subnormal_to_f64_rounding() {
    //     0.0 => 0
    //     let a = BigFloat {
    //         significand: BigInt::from_u64(0b100),
    //         exp: -1075,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res.to_bits(), 0b10);
    //     assert_eq!(res, 2.0f64.powf(-1073.0));
    //     assert!(res.is_subnormal());

    //     0.0+ => 0
    //     let a = BigFloat {
    //         significand: BigInt::from_u64(0b100),
    //         exp: -1075,
    //         non_zero_reminder: true,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res.to_bits(), 0b10);
    //     assert_eq!(res, 2.0f64.powf(-1073.0));
    //     assert!(res.is_subnormal());

    //     0.1 => 0
    //     let a = BigFloat {
    //         significand: BigInt::from_u64(0b101),
    //         exp: -1075,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res.to_bits(), 0b10);
    //     assert_eq!(res, 2.0f64.powf(-1073.0));
    //     assert!(res.is_subnormal());

    //     0.1+ => 1
    //     let a = BigFloat {
    //         significand: BigInt::from_u64(0b101),
    //         exp: -1075,
    //         non_zero_reminder: true,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res.to_bits(), 0b11);
    //     assert_eq!(res, 1.5f64 * 2.0f64.powf(-1073.0));
    //     assert!(res.is_subnormal());

    //     1.0 => 1
    //     let a = BigFloat {
    //         significand: BigInt::from_u64(0b110),
    //         exp: -1075,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res.to_bits(), 0b11);
    //     assert_eq!(res, 1.5f64 * 2.0f64.powf(-1073.0));
    //     assert!(res.is_subnormal());

    //     1.0+ => 1
    //     let a = BigFloat {
    //         significand: BigInt::from_u64(0b110),
    //         exp: -1075,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res.to_bits(), 0b11);
    //     assert_eq!(res, 1.5f64 * 2.0f64.powf(-1073.0));
    //     assert!(res.is_subnormal());

    //     1.1 => 2
    //     let a = BigFloat {
    //         significand: BigInt::from_u64(0b111),
    //         exp: -1075,
    //         non_zero_reminder: false,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res.to_bits(), 0b100);
    //     assert_eq!(res, 2.0f64.powf(-1072.0));
    //     assert!(res.is_subnormal());

    //     1.1+ => 2
    //     let a = BigFloat {
    //         significand: BigInt::from_u64(0b111),
    //         exp: -1075,
    //         non_zero_reminder: true,
    //     };
    //     let res = a.to_f64();
    //     assert_eq!(res.to_bits(), 0b100);
    //     assert_eq!(res, 2.0f64.powf(-1072.0));
    //     assert!(res.is_subnormal());
    // }

    // #[test]
    // #[wasm_bindgen_test]
    // fn test_rust_cast() {
    //     test(18014398509481981);
    //     test(18014398509481982);
    //     test(18014398509481983);
    //     test(18014398509481984);
    //     test(18014398509481985);

    //     fn test(n: u64) {
    //         let big_float = BigFloat {
    //             significand: BigInt::from_u64(n),
    //             exp: 0,
    //             non_zero_reminder: false,
    //         };
    //         let f64 = big_float.to_f64();
    //         assert_eq!(f64, n as f64);
    //     }
    // }
}

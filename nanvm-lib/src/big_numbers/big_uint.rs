use std::{
    cmp::Ordering,
    iter,
    ops::{Add, Div, Mul, Shl, Shr, Sub},
};

use crate::common::{cast::Cast, default::default, vec::new_resize};

use super::{big_int::BigInt, big_int::Sign};

#[derive(Debug, PartialEq, Clone, Eq, Default)]
pub struct BigUint {
    pub value: Vec<u64>,
}

impl BigUint {
    pub const ZERO: BigUint = BigUint { value: Vec::new() };

    pub fn one() -> BigUint {
        BigUint { value: [1].cast() }
    }

    pub fn normalize(&mut self) {
        while let Some(&0) = self.value.last() {
            self.value.pop();
        }
    }

    pub fn is_one(&self) -> bool {
        self.len() == 1 && self.value[0] == 1
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    // Clippy wants is_empty as soon as it sees len.
    // We want to use is_zero instead, but let's be respectful to Clippy anyway.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_zero(&self) -> bool {
        self.is_empty()
    }

    pub fn from_u64(n: u64) -> Self {
        BigUint { value: [n].cast() }
    }

    pub fn pow(&self, exp: &BigUint) -> BigUint {
        if self.is_one() {
            return BigUint::one();
        }

        if self.is_zero() {
            return if exp.is_zero() {
                BigUint::one()
            } else {
                BigUint::ZERO
            };
        }

        if exp.is_zero() {
            return BigUint::one();
        }

        if exp.len() != 1 {
            panic!("Maximum BigUint size exceeded")
        }

        self.pow_u64(exp.value[0])
    }

    pub fn pow_u64(&self, mut exp: u64) -> BigUint {
        let mut res = BigUint::one();
        let mut b = self.clone();
        loop {
            if exp == 0 {
                return res;
            }
            if exp & 1 > 0 {
                res = &res * &b;
            }
            exp >>= 1;
            b = &b * &b;
        }
    }

    pub fn get_last_bit(&self) -> u64 {
        if self.is_zero() {
            return 0;
        }

        self.value[0] & 1
    }

    pub fn to_big_int(self) -> BigInt {
        BigInt::new(Sign::Positive, self)
    }

    pub fn div_mod(&self, b: &Self) -> (BigUint, BigUint) {
        if b.is_zero() {
            panic!("attempt to divide by zero");
        }

        match self.cmp(b) {
            Ordering::Less => (default(), self.clone()),
            Ordering::Equal => (BigUint { value: [1].cast() }, default()),
            Ordering::Greater => {
                let mut a = self.clone();
                let mut result = BigUint::ZERO;
                loop {
                    if a.cmp(b) == Ordering::Less {
                        return (result, a);
                    }
                    let a_high_digit = a.len() - 1;
                    let b_high_digit = b.len() - 1;
                    let a_high = a.value[a_high_digit];
                    let b_high = b.value[b_high_digit];
                    let (q_index, q_digit) = match b_high.cmp(&a_high) {
                        Ordering::Less | Ordering::Equal => {
                            (a_high_digit - b_high_digit, a_high / b_high)
                        }
                        Ordering::Greater => {
                            let a_high_2 =
                                ((a_high as u128) << 64) + a.value[a_high_digit - 1] as u128;
                            (
                                a_high_digit - b_high_digit - 1,
                                (a_high_2 / b_high as u128) as u64,
                            )
                        }
                    };
                    let mut q = BigUint {
                        value: new_resize(q_index + 1),
                    };
                    q.value[q_index] = q_digit;
                    let mut m = b * &q;
                    if a.cmp(&m) == Ordering::Less {
                        q.value[q_index] = q_digit - 1;
                        m = b * &q;
                    }
                    a = &a - &m;
                    result = &result + &q;
                }
            }
        }
    }
}

impl Add for &BigUint {
    type Output = BigUint;

    fn add(self, other: Self) -> Self::Output {
        let mut value: Vec<_> = default();
        let mut carry = 0;
        let iter = match other.len() > self.len() {
            true => other
                .value
                .iter()
                .copied()
                .zip(self.value.iter().copied().chain(iter::repeat(0))),
            false => self
                .value
                .iter()
                .copied()
                .zip(other.value.iter().copied().chain(iter::repeat(0))),
        };
        for (a, b) in iter {
            let next = a as u128 + b as u128 + carry;
            value.push(next as u64);
            carry = next >> 64;
        }
        if carry != 0 {
            value.push(carry as u64);
        }
        BigUint { value }
    }
}

impl Sub for &BigUint {
    type Output = BigUint;

    fn sub(self, other: Self) -> Self::Output {
        match self.cmp(other) {
            Ordering::Less | Ordering::Equal => BigUint::ZERO,
            Ordering::Greater => {
                let mut value: Vec<_> = default();
                let mut borrow = 0;
                let iter = self
                    .value
                    .iter()
                    .copied()
                    .zip(other.value.iter().copied().chain(iter::repeat(0)));
                for (a, b) in iter {
                    let next = a as i128 - b as i128 - borrow;
                    value.push(next as u64);
                    borrow = next >> 64 & 1;
                }
                let mut res = BigUint { value };
                res.normalize();
                res
            }
        }
    }
}

impl PartialOrd for BigUint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BigUint {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_len = self.len();
        let other_len = other.len();
        if self_len != other_len {
            return self_len.cmp(&other_len);
        }

        for (self_digit, other_digit) in self
            .value
            .iter()
            .copied()
            .rev()
            .zip(other.value.iter().copied().rev())
        {
            if self_digit != other_digit {
                return self_digit.cmp(&other_digit);
            }
        }

        Ordering::Equal
    }
}

impl Mul for &BigUint {
    type Output = BigUint;

    fn mul(self, other: Self) -> Self::Output {
        if self.is_zero() || other.is_zero() {
            return BigUint::ZERO;
        }

        let lhs_max = self.len() - 1;
        let rhs_max = other.len() - 1;
        let total_max = self.len() + other.len() - 1;
        let mut value = new_resize(total_max + 1);
        let mut i: usize = 0;
        while i < total_max {
            let mut j = i.saturating_sub(rhs_max);
            let max = if i < lhs_max { i } else { lhs_max };
            while j <= max {
                value = add_to_vec(value, i, self.value[j] as u128 * other.value[i - j] as u128);
                j += 1;
            }
            i += 1;
        }

        let mut result = BigUint { value };
        result.normalize();
        result
    }
}

impl Div for &BigUint {
    type Output = BigUint;

    fn div(self, b: Self) -> Self::Output {
        if b.is_zero() {
            panic!("attempt to divide by zero");
        }

        let (res, _) = self.div_mod(b);
        res
    }
}

impl Shl for &BigUint {
    type Output = BigUint;

    fn shl(self, rhs: Self) -> Self::Output {
        if self.is_zero() | rhs.is_zero() {
            return self.clone();
        }

        if rhs.len() != 1 {
            panic!("Maximum BigUint size exceeded")
        }

        let mut value = self.value.clone();
        let shift_mod = rhs.value[0] & ((1 << 6) - 1);
        if shift_mod > 0 {
            let len = value.len();
            value.push(0); //todo: check if it is neccessary?
            for i in (0..=len - 1).rev() {
                let mut digit = value[i] as u128;
                digit <<= shift_mod;
                value[i + 1] |= (digit >> 64) as u64;
                value[i] = digit as u64;
            }
        }

        let number_of_zeros = (rhs.value[0] / 64) as usize;
        if number_of_zeros > 0 {
            let mut zeros_vector: Vec<_> = new_resize(number_of_zeros);
            zeros_vector.extend(value);
            value = zeros_vector;
        }

        let mut res = BigUint { value };
        res.normalize();
        res
    }
}

impl Shr for &BigUint {
    type Output = BigUint;

    fn shr(self, rhs: Self) -> Self::Output {
        if self.is_zero() | rhs.is_zero() {
            return self.clone();
        }

        let number_to_remove = (rhs.value[0] / 64) as usize;
        if number_to_remove >= self.len() {
            return BigUint::ZERO;
        }

        let mut value = self.value.clone();
        value = value.split_off(number_to_remove);
        let shift_mod = rhs.value[0] & ((1 << 6) - 1);
        if shift_mod > 0 {
            let len = value.len();
            let mask = 1 << (shift_mod - 1);
            let mut i = 0;
            loop {
                value[i] >>= shift_mod;
                i += 1;
                if i == len {
                    break;
                }
                value[i - 1] |= (value[i] & mask) << (64 - shift_mod);
            }
        }

        let mut res = BigUint { value };
        res.normalize();
        res
    }
}

fn add_to_vec(mut vec: Vec<u64>, index: usize, add: u128) -> Vec<u64> {
    let sum = vec[index] as u128 + add;
    vec[index] = sum as u64;
    let carry = sum >> 64;
    if carry > 0 {
        vec = add_to_vec(vec, index + 1, carry);
    }
    vec
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::common::cast::Cast;

    use super::BigUint;

    #[test]
    #[wasm_bindgen_test]
    fn test_ord() {
        let a = BigUint { value: [1].cast() };
        let b = BigUint { value: [1].cast() };
        assert_eq!(a.cmp(&b), Ordering::Equal);

        let a = BigUint { value: [1].cast() };
        let b = BigUint { value: [2].cast() };
        assert_eq!(a.cmp(&b), Ordering::Less);

        let a = BigUint { value: [2].cast() };
        let b = BigUint { value: [1].cast() };
        assert_eq!(a.cmp(&b), Ordering::Greater);

        let a = BigUint {
            value: [1, 2].cast(),
        };
        let b = BigUint {
            value: [2, 1].cast(),
        };
        assert_eq!(a.cmp(&b), Ordering::Greater);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_add() {
        let a = BigUint { value: [1].cast() };
        let b = BigUint { value: [2].cast() };
        let result = &a + &b;
        assert_eq!(&result, &BigUint { value: [3].cast() });

        let a = BigUint { value: [1].cast() };
        let b = BigUint {
            value: [2, 4].cast(),
        };
        let result = &a + &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [3, 4].cast()
            }
        );

        let a = BigUint {
            value: [1 << 63].cast(),
        };
        let b = BigUint {
            value: [1 << 63].cast(),
        };
        let result = &a + &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [0, 1].cast()
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_add_overflow() {
        let a = BigUint {
            value: [u64::MAX, 0, 1].cast(),
        };
        let b = BigUint {
            value: [u64::MAX, u64::MAX].cast(),
        };
        let result = &a + &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [u64::MAX - 1, 0, 2].cast()
            }
        );
        let result = &b + &a;
        assert_eq!(
            &result,
            &BigUint {
                value: [u64::MAX - 1, 0, 2].cast()
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_sub() {
        let a = BigUint {
            value: [1 << 63].cast(),
        };
        let b = BigUint {
            value: [1 << 63].cast(),
        };
        let result = &a - &b;
        assert_eq!(&result, &BigUint::ZERO);

        let a = BigUint { value: [3].cast() };
        let b = BigUint { value: [2].cast() };
        let result = &a - &b;
        assert_eq!(&result, &BigUint { value: [1].cast() });
        let result = &b - &a;
        assert_eq!(&result, &BigUint::ZERO);

        let a = BigUint {
            value: [0, 1].cast(),
        };
        let b = BigUint { value: [1].cast() };
        let result = &a - &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [u64::MAX].cast()
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_mul() {
        let a = BigUint { value: [1].cast() };
        let result = &a * &BigUint::ZERO;
        assert_eq!(&result, &BigUint::ZERO);
        let result = &BigUint::ZERO * &a;
        assert_eq!(&result, &BigUint::ZERO);

        let a = BigUint { value: [1].cast() };
        let result = &a * &a;
        assert_eq!(&result, &a);

        let a = BigUint {
            value: [1, 2, 3, 4].cast(),
        };
        let b = BigUint {
            value: [5, 6, 7].cast(),
        };
        let result = &a * &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [5, 16, 34, 52, 45, 28].cast()
            },
        );
        let result = &b * &a;
        assert_eq!(
            &result,
            &BigUint {
                value: [5, 16, 34, 52, 45, 28].cast()
            },
        );

        let a = BigUint {
            value: [u64::MAX].cast(),
        };
        let b = BigUint {
            value: [u64::MAX].cast(),
        };
        let result = &a * &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [1, u64::MAX - 1].cast()
            },
        );
        let result = &b * &a;
        assert_eq!(
            &result,
            &BigUint {
                value: [1, u64::MAX - 1].cast()
            },
        );

        let a = BigUint {
            value: [u64::MAX, u64::MAX, u64::MAX].cast(),
        };
        let b = BigUint {
            value: [u64::MAX].cast(),
        };
        let result = &a * &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [1, u64::MAX, u64::MAX, u64::MAX - 1].cast()
            },
        );
        let result = &b * &a;
        assert_eq!(
            &result,
            &BigUint {
                value: [1, u64::MAX, u64::MAX, u64::MAX - 1].cast()
            },
        );
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    #[wasm_bindgen_test]
    fn test_div_by_zero() {
        let a = BigUint { value: [1].cast() };
        let _result = &a / &BigUint::ZERO;
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    #[wasm_bindgen_test]
    fn test_div_zero_by_zero() {
        let _result = &BigUint::ZERO / &BigUint::ZERO;
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_div_simple() {
        let a = BigUint { value: [2].cast() };
        let b = BigUint { value: [7].cast() };
        let result = &a / &b;
        assert_eq!(&result, &BigUint::ZERO);

        let a = BigUint { value: [7].cast() };
        let result = &a / &a;
        assert_eq!(&result, &BigUint { value: [1].cast() });

        let a = BigUint { value: [7].cast() };
        let b = BigUint { value: [2].cast() };
        let result = &a / &b;
        assert_eq!(&result, &BigUint { value: [3].cast() });

        let a = BigUint {
            value: [6, 8].cast(),
        };
        let b = BigUint { value: [2].cast() };
        let result = &a / &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [3, 4].cast()
            }
        );

        let a = BigUint {
            value: [4, 7].cast(),
        };
        let b = BigUint { value: [2].cast() };
        let result = &a / &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [(1 << 63) + 2, 3].cast()
            }
        );

        let a = BigUint {
            value: [0, 4].cast(),
        };
        let b = BigUint {
            value: [1, 2].cast(),
        };
        let result = &a / &b;
        assert_eq!(&result, &BigUint { value: [1].cast() });

        let a = BigUint {
            value: [1, 1].cast(),
        };
        let b = BigUint { value: [1].cast() };
        let result = &a / &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [1, 1].cast()
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_div_mod() {
        let a = BigUint { value: [7].cast() };
        let b = BigUint { value: [2].cast() };
        let result = a.div_mod(&b);
        assert_eq!(
            result,
            (BigUint { value: [3].cast() }, BigUint { value: [1].cast() })
        );

        let a = BigUint {
            value: [7, 5].cast(),
        };
        let b = BigUint {
            value: [0, 3].cast(),
        };
        let result = a.div_mod(&b);
        assert_eq!(
            result,
            (
                BigUint { value: [1].cast() },
                BigUint {
                    value: [7, 2].cast()
                }
            )
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_pow_u64() {
        let a = BigUint {
            value: [100].cast(),
        };
        let result = a.pow_u64(0);
        assert_eq!(&result, &BigUint { value: [1].cast() });

        let a = BigUint { value: [2].cast() };
        let result = a.pow_u64(7);
        assert_eq!(
            &result,
            &BigUint {
                value: [128].cast()
            }
        );

        let a = BigUint { value: [5].cast() };
        let result = a.pow_u64(3);
        assert_eq!(
            &result,
            &BigUint {
                value: [125].cast()
            }
        );

        let a = BigUint::ZERO;
        let result = a.pow_u64(3);
        assert_eq!(&result, &BigUint::ZERO);

        let a = BigUint::ZERO;
        let result = a.pow_u64(0);
        assert_eq!(&result, &BigUint { value: [1].cast() });

        let a = BigUint::one();
        let result = a.pow_u64(0);
        assert_eq!(&result, &BigUint { value: [1].cast() });

        let a = BigUint::one();
        let result = a.pow_u64(100);
        assert_eq!(&result, &BigUint { value: [1].cast() });
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_pow() {
        let a = BigUint {
            value: [100].cast(),
        };
        let result = a.pow(&BigUint::ZERO);
        assert_eq!(&result, &BigUint { value: [1].cast() });

        let a = BigUint { value: [2].cast() };
        let result = a.pow(&BigUint { value: [7].cast() });
        assert_eq!(
            &result,
            &BigUint {
                value: [128].cast()
            }
        );

        let a = BigUint { value: [5].cast() };
        let result = a.pow(&BigUint { value: [3].cast() });
        assert_eq!(
            &result,
            &BigUint {
                value: [125].cast()
            }
        );

        let a = BigUint::ZERO;
        let result = a.pow(&BigUint {
            value: [100, 100].cast(),
        });
        assert_eq!(&result, &BigUint::ZERO);

        let a = BigUint::ZERO;
        let result = a.pow(&BigUint::ZERO);
        assert_eq!(&result, &BigUint { value: [1].cast() });

        let a = BigUint::one();
        let result = a.pow(&BigUint::ZERO);
        assert_eq!(&result, &BigUint { value: [1].cast() });

        let a = BigUint::one();
        let result = a.pow(&BigUint {
            value: [100, 100].cast(),
        });
        assert_eq!(&result, &BigUint { value: [1].cast() });
    }

    #[test]
    #[should_panic(expected = "Maximum BigUint size exceeded")]
    #[wasm_bindgen_test]
    fn test_pow_overflow() {
        let a = BigUint { value: [5].cast() };
        let _result = a.pow(&BigUint {
            value: [100, 100].cast(),
        });
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_shl_zero() {
        let result = &BigUint::ZERO << &BigUint::ZERO;
        assert_eq!(&result, &BigUint::ZERO);

        let a = BigUint { value: [5].cast() };
        let result = &a << &BigUint::ZERO;
        assert_eq!(result, a);

        let result = &BigUint::ZERO << &a;
        assert_eq!(result, BigUint::ZERO);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_shl() {
        let a = BigUint { value: [1].cast() };
        let result = &a << &a;
        assert_eq!(result, BigUint { value: [2].cast() });

        let a = BigUint { value: [5].cast() };
        let b = BigUint { value: [63].cast() };
        let result = &a << &b;
        assert_eq!(
            result,
            BigUint {
                value: [1 << 63, 2].cast()
            }
        );

        let a = BigUint {
            value: [5, 9].cast(),
        };
        let b = BigUint { value: [63].cast() };
        let result = &a << &b;
        assert_eq!(
            result,
            BigUint {
                value: [1 << 63, (1 << 63) + 2, 4].cast()
            }
        );

        let a = BigUint {
            value: [5, 9].cast(),
        };
        let b = BigUint { value: [64].cast() };
        let result = &a << &b;
        assert_eq!(
            result,
            BigUint {
                value: [0, 5, 9].cast()
            }
        );

        let a = BigUint {
            value: [5, 9].cast(),
        };
        let b = BigUint { value: [65].cast() };
        let result = &a << &b;
        assert_eq!(
            result,
            BigUint {
                value: [0, 10, 18].cast()
            }
        );
    }

    #[test]
    #[should_panic(expected = "Maximum BigUint size exceeded")]
    #[wasm_bindgen_test]
    fn test_shl_overflow() {
        let a = BigUint::one();
        let b = BigUint {
            value: [1, 1].cast(),
        };
        let _result = &a << &b;
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_shr_zero() {
        let result = &BigUint::ZERO >> &BigUint::ZERO;
        assert_eq!(&result, &BigUint::ZERO);

        let a = BigUint { value: [5].cast() };
        let result = &a >> &BigUint::ZERO;
        assert_eq!(result, a);

        let result = &BigUint::ZERO >> &a;
        assert_eq!(result, BigUint::ZERO);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_shr() {
        let a = BigUint {
            value: [1, 1, 1, 1].cast(),
        };
        let b = BigUint {
            value: [256].cast(),
        };
        let result = &a >> &b;
        assert_eq!(result, BigUint::ZERO);

        let a = BigUint { value: [1].cast() };
        let result = &a >> &a;
        assert_eq!(result, BigUint::ZERO);

        let a = BigUint { value: [2].cast() };
        let b = BigUint { value: [1].cast() };
        let result = &a >> &b;
        assert_eq!(result, BigUint { value: [1].cast() });

        let a = BigUint {
            value: [1, 5, 9].cast(),
        };
        let b = BigUint { value: [65].cast() };
        let result = &a >> &b;
        assert_eq!(
            result,
            BigUint {
                value: [(1 << 63) + 2, 4].cast()
            }
        );
    }
}

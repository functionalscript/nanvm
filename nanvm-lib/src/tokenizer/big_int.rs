use std::{
    cmp::Ordering,
    iter,
    ops::{Add, Div, Mul, Neg, Sub},
};

use crate::common::{array::ArrayEx, default::default};
use crate::tokenizer::big_uint::BigUint;

#[derive(Debug, PartialEq, Clone, Eq, Default)]
struct BigInt {
    sign: Sign,
    value: Vec<u64>,
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
enum Sign {
    Positive = 1,
    Negative = -1,
}

impl Default for Sign {
    fn default() -> Self {
        Self::Positive
    }
}

impl BigInt {
    const ZERO: BigInt = BigInt {
        sign: Sign::Positive,
        value: Vec::new(),
    };

    fn normalize(&mut self) {
        loop {
            match self.value.last() {
                Some(&last) if last == 0 => {
                    self.value.pop();
                }
                _ => break,
            }
        }
    }

    fn is_zero(&self) -> bool {
        self.value.len() == 0
    }
}

impl Add for &BigInt {
    type Output = BigInt;

    fn add(self, other: Self) -> Self::Output {
        match self.sign == other.sign {
            true => add_same_sign(self.sign, &self.value, &other.value),
            false => match cmp_values(&self.value, &other.value) {
                Ordering::Equal => BigInt::ZERO,
                Ordering::Greater => substract_same_sign(self.sign, &self.value, &other.value),
                Ordering::Less => substract_same_sign(other.sign, &other.value, &self.value),
            },
        }
    }
}

impl Sub for BigInt {
    type Output = BigInt;

    fn sub(self, other: Self) -> Self::Output {
        self.add(&-other)
    }
}

impl Neg for BigInt {
    type Output = BigInt;

    fn neg(mut self) -> Self::Output {
        if self.value.is_empty() {
            return self;
        }
        self.sign = if self.sign == Sign::Positive {
            Sign::Negative
        } else {
            Sign::Positive
        };
        self
    }
}

impl PartialOrd for Sign {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Sign {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self as i8).cmp(&(*other as i8))
    }
}

impl PartialOrd for BigInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BigInt {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.sign != other.sign {
            return self.sign.cmp(&other.sign);
        }

        cmp_values(&self.value, &other.value)
    }
}

impl Mul for &BigInt {
    type Output = BigInt;

    fn mul(self, other: Self) -> Self::Output {
        if self.is_zero() || other.is_zero() {
            return BigInt::ZERO;
        }
        let bu = &BigUint {
            value: self.value.clone(),
        } * &BigUint {
            value: other.value.clone(),
        };
        let sign = match self.sign == other.sign {
            true => Sign::Positive,
            false => Sign::Negative,
        };
        let mut result = BigInt {
            sign,
            value: bu.value,
        };
        result.normalize();
        result
    }
}

impl Div for &BigInt {
    type Output = BigInt;

    fn div(self, d: Self) -> Self::Output {
        if d.is_zero() {
            panic!("attempt to divide by zero");
        }

        let bu = BigUint {
            value: self.value.clone(),
        } / BigUint {
            value: d.value.clone(),
        };
        match self.sign == d.sign {
            true => BigInt {
                sign: Sign::Positive,
                value: bu.value,
            },
            false => BigInt {
                sign: Sign::Negative,
                value: bu.value,
            },
        }
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

fn cmp_values(lhs: &Vec<u64>, rhs: &Vec<u64>) -> Ordering {
    let lhs_len = lhs.len();
    let rhs_len = rhs.len();
    if lhs_len != rhs_len {
        return lhs_len.cmp(&rhs_len);
    }

    for (lhs_digit, rhs_digit) in lhs.iter().copied().rev().zip(rhs.iter().copied().rev()) {
        if lhs_digit != rhs_digit {
            return lhs_digit.cmp(&rhs_digit);
        }
    }

    Ordering::Equal
}

fn add_same_sign(sign: Sign, lhs: &Vec<u64>, rhs: &Vec<u64>) -> BigInt {
    let mut value: Vec<_> = default();
    let mut carry = 0;
    let iter = match rhs.len() > lhs.len() {
        true => rhs
            .iter()
            .copied()
            .zip(lhs.iter().copied().chain(iter::repeat(0))),
        false => lhs
            .iter()
            .copied()
            .zip(rhs.iter().copied().chain(iter::repeat(0))),
    };
    for (a, b) in iter {
        let next = a as u128 + b as u128 + carry;
        value.push(next as u64);
        carry = next >> 64;
    }
    if carry != 0 {
        value.push(carry as u64);
    }
    BigInt { sign, value }
}

fn substract_same_sign(sign: Sign, lhs: &Vec<u64>, rhs: &Vec<u64>) -> BigInt {
    let mut value = substract_vec(lhs, rhs);
    let mut result = BigInt { sign, value };
    result.normalize();
    result
}

fn substract_vec(lhs: &Vec<u64>, rhs: &Vec<u64>) -> Vec<u64> {
    let mut value: Vec<_> = default();
    let mut borrow = 0;
    let iter = lhs
        .iter()
        .copied()
        .zip(rhs.iter().copied().chain(iter::repeat(0)));
    for (a, b) in iter {
        let next = a as i128 - b as i128 - borrow;
        value.push(next as u64);
        borrow = next >> 64 & 1;
    }
    value
}

#[cfg(test)]
mod test {
    use std::{cmp::Ordering, default};

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::common::array::ArrayEx;

    use super::{BigInt, Sign};

    #[test]
    #[wasm_bindgen_test]
    fn test_ord() {
        let a = BigInt {
            sign: Sign::Positive,
            value: [1].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [1].vec(),
        };
        assert_eq!(a.cmp(&b), Ordering::Equal);

        let a = BigInt {
            sign: Sign::Positive,
            value: [1].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [2].vec(),
        };
        assert_eq!(a.cmp(&b), Ordering::Less);

        let a = BigInt {
            sign: Sign::Positive,
            value: [1].vec(),
        };
        let b = BigInt {
            sign: Sign::Negative,
            value: [2].vec(),
        };
        assert_eq!(a.cmp(&b), Ordering::Greater);

        let a = BigInt {
            sign: Sign::Positive,
            value: [1, 2].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [2, 1].vec(),
        };
        assert_eq!(a.cmp(&b), Ordering::Greater);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_add_same_sign() {
        let a = BigInt {
            sign: Sign::Positive,
            value: [1].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [2].vec(),
        };
        let result = &a + &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [3].vec()
            }
        );

        let a = BigInt {
            sign: Sign::Negative,
            value: [1].vec(),
        };
        let b = BigInt {
            sign: Sign::Negative,
            value: [2].vec(),
        };
        let result = &a + &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Negative,
                value: [3].vec()
            }
        );

        let a = BigInt {
            sign: Sign::Positive,
            value: [1].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [2, 4].vec(),
        };
        let result = &a + &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [3, 4].vec()
            }
        );

        let a = BigInt {
            sign: Sign::Positive,
            value: [1 << 63].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [1 << 63].vec(),
        };
        let result = &a + &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [0, 1].vec()
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_add_different_sign() {
        let a = BigInt {
            sign: Sign::Positive,
            value: [1 << 63].vec(),
        };
        let b = BigInt {
            sign: Sign::Negative,
            value: [1 << 63].vec(),
        };
        let result = &a + &b;
        assert_eq!(&result, &BigInt::ZERO);

        let a = BigInt {
            sign: Sign::Positive,
            value: [3].vec(),
        };
        let b = BigInt {
            sign: Sign::Negative,
            value: [2].vec(),
        };
        let result = &a + &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [1].vec()
            }
        );
        let result = &b + &a;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [1].vec()
            }
        );

        let a = BigInt {
            sign: Sign::Positive,
            value: [0, 1].vec(),
        };
        let b = BigInt {
            sign: Sign::Negative,
            value: [1].vec(),
        };
        let result = &a + &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [u64::MAX].vec()
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_add_overflow() {
        let a = BigInt {
            sign: Sign::Positive,
            value: [u64::MAX, 0, 1].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [u64::MAX, u64::MAX].vec(),
        };
        let result = &a + &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [u64::MAX - 1, 0, 2].vec()
            }
        );
        let result = &b + &a;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [u64::MAX - 1, 0, 2].vec()
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_sub_same_sign() {
        let a = BigInt {
            sign: Sign::Positive,
            value: [1 << 63].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [1 << 63].vec(),
        };
        let result = a - b;
        assert_eq!(&result, &BigInt::ZERO);

        let a = BigInt {
            sign: Sign::Positive,
            value: [3].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [2].vec(),
        };
        let result = a.clone() - b.clone();
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [1].vec()
            }
        );
        let result = b - a;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Negative,
                value: [1].vec()
            }
        );

        let a = BigInt {
            sign: Sign::Positive,
            value: [0, 1].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [1].vec(),
        };
        let result = a - b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [u64::MAX].vec()
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_sub_different_sign() {
        let a = BigInt {
            sign: Sign::Positive,
            value: [1].vec(),
        };
        let b = BigInt {
            sign: Sign::Negative,
            value: [2].vec(),
        };
        let result = a - b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [3].vec()
            }
        );

        let a = BigInt {
            sign: Sign::Negative,
            value: [1].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [2].vec(),
        };
        let result = a - b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Negative,
                value: [3].vec()
            }
        );

        let a = BigInt {
            sign: Sign::Positive,
            value: [1].vec(),
        };
        let b = BigInt {
            sign: Sign::Negative,
            value: [2, 4].vec(),
        };
        let result = a - b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [3, 4].vec()
            }
        );

        let a = BigInt {
            sign: Sign::Positive,
            value: [1 << 63].vec(),
        };
        let b = BigInt {
            sign: Sign::Negative,
            value: [1 << 63].vec(),
        };
        let result = a - b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [0, 1].vec()
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_mul() {
        let a = BigInt {
            sign: Sign::Positive,
            value: [1].vec(),
        };
        let result = &a * &BigInt::ZERO;
        assert_eq!(&result, &BigInt::ZERO);
        let result = &BigInt::ZERO * &a;
        assert_eq!(&result, &BigInt::ZERO);

        let a = BigInt {
            sign: Sign::Negative,
            value: [1].vec(),
        };
        let result = &a * &BigInt::ZERO;
        assert_eq!(&result, &BigInt::ZERO);
        let result = &BigInt::ZERO * &a;
        assert_eq!(&result, &BigInt::ZERO);

        let a = BigInt {
            sign: Sign::Negative,
            value: [1].vec(),
        };
        let b = BigInt {
            sign: Sign::Negative,
            value: [1].vec(),
        };
        let result = &a * &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [1].vec()
            },
        );

        let a = BigInt {
            sign: Sign::Negative,
            value: [1, 2, 3, 4].vec(),
        };
        let b = BigInt {
            sign: Sign::Negative,
            value: [5, 6, 7].vec(),
        };
        let result = &a * &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [5, 16, 34, 52, 45, 28].vec()
            },
        );
        let result = &b * &a;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [5, 16, 34, 52, 45, 28].vec()
            },
        );

        let a = BigInt {
            sign: Sign::Negative,
            value: [u64::MAX].vec(),
        };
        let b = BigInt {
            sign: Sign::Negative,
            value: [u64::MAX].vec(),
        };
        let result = &a * &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [1, u64::MAX - 1].vec()
            },
        );
        let result = &b * &a;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [1, u64::MAX - 1].vec()
            },
        );

        let a = BigInt {
            sign: Sign::Negative,
            value: [u64::MAX, u64::MAX, u64::MAX].vec(),
        };
        let b = BigInt {
            sign: Sign::Negative,
            value: [u64::MAX].vec(),
        };
        let result = &a * &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [1, u64::MAX, u64::MAX, u64::MAX - 1].vec()
            },
        );
        let result = &b * &a;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [1, u64::MAX, u64::MAX, u64::MAX - 1].vec()
            },
        );
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    #[wasm_bindgen_test]
    fn test_div_by_zero() {
        let a = BigInt {
            sign: Sign::Positive,
            value: [1].vec(),
        };
        let result = &a / &BigInt::ZERO;
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    #[wasm_bindgen_test]
    fn test_div_zero_by_zero() {
        let result = &BigInt::ZERO / &BigInt::ZERO;
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_div_simple() {
        let a = BigInt {
            sign: Sign::Positive,
            value: [2].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [7].vec(),
        };
        let result = &a / &b;
        assert_eq!(&result, &BigInt::ZERO);

        let a = BigInt {
            sign: Sign::Positive,
            value: [7].vec(),
        };
        let result = &a / &a;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [1].vec()
            }
        );

        let a = BigInt {
            sign: Sign::Positive,
            value: [7].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [2].vec(),
        };
        let result = &a / &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [3].vec()
            }
        );

        let a = BigInt {
            sign: Sign::Positive,
            value: [6, 8].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [2].vec(),
        };
        let result = &a / &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [3, 4].vec()
            }
        );

        let a = BigInt {
            sign: Sign::Positive,
            value: [4, 7].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [2].vec(),
        };
        let result = &a / &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: [(1 << 63) + 2, 3].vec()
            }
        );
    }
}

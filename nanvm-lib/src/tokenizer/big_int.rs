use std::{cmp::Ordering, iter, ops::Add};

use crate::common::default::default;

#[derive(Debug, PartialEq, Eq)]
struct BigInt {
    sign: Sign,
    value: Vec<u64>,
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
enum Sign {
    Positive = 1,
    Negative = -1,
}

impl BigInt {
    fn normalize(mut self) {
        todo!()
    }
}

impl Add for &BigInt {
    type Output = BigInt;

    fn add(self, other: Self) -> Self::Output {
        match self.sign == other.sign {
            true => add_same_sign(self.sign, &self.value, &other.value),
            false => match cmp_values(&self.value, &other.value) {
                Ordering::Equal => BigInt {
                    sign: Sign::Positive,
                    value: default(),
                },
                Ordering::Greater => substract_same_sign(self.sign, &self.value, &other.value),
                Ordering::Less => substract_same_sign(other.sign, &other.value, &self.value),
            },
        }
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
    let mut value: Vec<_> = default();
    let mut borrow = 0;
    let mut iter = lhs
        .iter()
        .copied()
        .zip(rhs.iter().copied().chain(iter::repeat(0)));
    for (a, b) in iter {
        let next = a as i128 - b as i128 - borrow;
        value.push(next as u64);
        borrow = next >> 64 & 1;
    }
    BigInt { sign, value }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::common::array::ArrayEx;

    use crate::common::default::default;

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
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: default()
            }
        );

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
}

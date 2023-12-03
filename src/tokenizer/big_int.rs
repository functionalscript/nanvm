use std::{iter, ops::Add, cmp::Ordering};

use crate::common::default::default;

#[derive(Debug, PartialEq, Eq)]
struct BigInt {
    sign: Sign,
    value: Vec<u64>,
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
enum Sign {
    Positive,
    Negative,
}

impl Add for BigInt {
    type Output = BigInt;

    fn add(self, other: Self) -> Self::Output {
        match self.sign == other.sign {
            true => add_same_sign(self, other),
            false => substract_same_sign(self, other),
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
        match (self, other) {
            (Sign::Positive, Sign::Negative) => Ordering::Greater,
            (Sign::Negative, Sign::Positive) => Ordering::Less,
            _ => Ordering::Equal
        }
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

        let self_len = self.value.len();
        let other_len = other.value.len();
        if self_len != other_len {
            return self_len.cmp(&other_len);
        }

        for (&self_digit, &other_digit) in self.value.iter().rev().zip(other.value.iter().rev()) {
            if self_digit != other_digit {
                return self_digit.cmp(&other_digit);
            }
        }

        Ordering::Equal
    }
}

fn add_same_sign(lhs: BigInt, rhs: BigInt) -> BigInt {
    let mut result: Vec<_> = default();
    let mut carry = 0;
    let iter = match rhs.value.len() > lhs.value.len() {
        true => rhs
            .value
            .iter()
            .zip(lhs.value.iter().chain(iter::repeat(&0))),
        false => lhs
            .value
            .iter()
            .zip(rhs.value.iter().chain(iter::repeat(&0))),
    };
    for (a, b) in iter {
        let next = a.wrapping_add(carry).wrapping_add(*b);
        result.push(next);
        carry = if next < *a { 1 } else { 0 };
    }
    if carry == 1 {
        result.push(1);
    }
    BigInt {
        sign: lhs.sign,
        value: result,
    }
}

fn substract_same_sign(lhs: BigInt, rhs: BigInt) -> BigInt {
    todo!()
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

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
    fn test_add() {
        let a = BigInt {
            sign: Sign::Positive,
            value: [1].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [2].vec(),
        };
        let result = a + b;
        assert_eq!(&result, &BigInt { sign: Sign::Positive, value: [3].vec()});

        let a = BigInt {
            sign: Sign::Negative,
            value: [1].vec(),
        };
        let b = BigInt {
            sign: Sign::Negative,
            value: [2].vec(),
        };
        let result = a + b;
        assert_eq!(&result, &BigInt { sign: Sign::Negative, value: [3].vec()});

        let a = BigInt {
            sign: Sign::Positive,
            value: [1].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [2, 4].vec(),
        };
        let result = a + b;
        assert_eq!(&result, &BigInt { sign: Sign::Positive, value: [3, 4].vec()});

        let a = BigInt {
            sign: Sign::Positive,
            value: [1 << 63].vec(),
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: [1 << 63].vec(),
        };
        let result = a + b;
        assert_eq!(&result, &BigInt { sign: Sign::Positive, value: [0, 1].vec()});
    }
}

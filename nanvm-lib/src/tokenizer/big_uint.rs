use std::{
    cmp::Ordering,
    iter,
    ops::{Add, Div, Mul, Sub},
};

use crate::common::{array::ArrayEx, default::default};

#[derive(Debug, PartialEq, Clone, Eq, Default)]
pub struct BigUint {
    pub value: Vec<u64>,
}

impl BigUint {
    pub const ZERO: BigUint = BigUint { value: Vec::new() };

    pub fn normalize(&mut self) {
        loop {
            match self.value.last() {
                Some(&last) if last == 0 => {
                    self.value.pop();
                }
                _ => break,
            }
        }
    }

    pub fn is_zero(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    fn div_mod(&self, b: &Self) -> (BigUint, BigUint) {
        if b.is_zero() {
            panic!("attempt to divide by zero");
        }

        match self.cmp(&b) {
            Ordering::Less => (default(), self.clone()),
            Ordering::Equal => (BigUint { value: [1].vec() }, default()),
            Ordering::Greater => {
                let mut a = self.clone();
                let mut result = BigUint::ZERO;
                loop {
                    if a.cmp(&b) == Ordering::Less {
                        return (result, a);
                    }
                    let a_high_digit = a.len() - 1;
                    let b_high_digit = b.len() - 1;
                    let a_high = a.value[a_high_digit];
                    let b_high = b.value[b_high_digit];
                    match b_high.cmp(&a_high) {
                        Ordering::Less | Ordering::Equal => {
                            let q_index = a_high_digit - b_high_digit;
                            let q_digit = a_high / b_high;
                            let mut q = BigUint {
                                value: vec![0; q_index as usize + 1],
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
                        Ordering::Greater => {
                            let a_high_2 =
                                ((a_high as u128) << 64) + a.value[a_high_digit - 1] as u128;
                            let q_index = a_high_digit - b_high_digit - 1;
                            let q_digit = (a_high_2 / b_high as u128) as u64;
                            let mut q = BigUint {
                                value: vec![0; q_index as usize + 1],
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
        match self.cmp(&other) {
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
        let mut value = vec![0; total_max + 1];
        let mut i: usize = 0;
        while i < total_max {
            let mut j = if i > rhs_max { i - rhs_max } else { 0 };
            let max = if i < lhs_max { i } else { lhs_max };
            while j <= max {
                value = add_to_vec(value, i, self.value[j] as u128 * other.value[i - j] as u128);
                j = j + 1;
            }
            i = i + 1;
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
    use std::{cmp::Ordering, default};

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::common::array::ArrayEx;

    use super::BigUint;

    #[test]
    #[wasm_bindgen_test]
    fn test_ord() {
        let a = BigUint { value: [1].vec() };
        let b = BigUint { value: [1].vec() };
        assert_eq!(a.cmp(&b), Ordering::Equal);

        let a = BigUint { value: [1].vec() };
        let b = BigUint { value: [2].vec() };
        assert_eq!(a.cmp(&b), Ordering::Less);

        let a = BigUint { value: [2].vec() };
        let b = BigUint { value: [1].vec() };
        assert_eq!(a.cmp(&b), Ordering::Greater);

        let a = BigUint {
            value: [1, 2].vec(),
        };
        let b = BigUint {
            value: [2, 1].vec(),
        };
        assert_eq!(a.cmp(&b), Ordering::Greater);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_add() {
        let a = BigUint { value: [1].vec() };
        let b = BigUint { value: [2].vec() };
        let result = &a + &b;
        assert_eq!(&result, &BigUint { value: [3].vec() });

        let a = BigUint { value: [1].vec() };
        let b = BigUint {
            value: [2, 4].vec(),
        };
        let result = &a + &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [3, 4].vec()
            }
        );

        let a = BigUint {
            value: [1 << 63].vec(),
        };
        let b = BigUint {
            value: [1 << 63].vec(),
        };
        let result = &a + &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [0, 1].vec()
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_add_overflow() {
        let a = BigUint {
            value: [u64::MAX, 0, 1].vec(),
        };
        let b = BigUint {
            value: [u64::MAX, u64::MAX].vec(),
        };
        let result = &a + &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [u64::MAX - 1, 0, 2].vec()
            }
        );
        let result = &b + &a;
        assert_eq!(
            &result,
            &BigUint {
                value: [u64::MAX - 1, 0, 2].vec()
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_sub() {
        let a = BigUint {
            value: [1 << 63].vec(),
        };
        let b = BigUint {
            value: [1 << 63].vec(),
        };
        let result = &a - &b;
        assert_eq!(&result, &BigUint::ZERO);

        let a = BigUint { value: [3].vec() };
        let b = BigUint { value: [2].vec() };
        let result = &a - &b;
        assert_eq!(&result, &BigUint { value: [1].vec() });
        let result = &b - &a;
        assert_eq!(&result, &BigUint::ZERO);

        let a = BigUint {
            value: [0, 1].vec(),
        };
        let b = BigUint { value: [1].vec() };
        let result = &a - &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [u64::MAX].vec()
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_mul() {
        let a = BigUint { value: [1].vec() };
        let result = &a * &BigUint::ZERO;
        assert_eq!(&result, &BigUint::ZERO);
        let result = &BigUint::ZERO * &a;
        assert_eq!(&result, &BigUint::ZERO);

        let a = BigUint { value: [1].vec() };
        let result = &a * &a;
        assert_eq!(&result, &a);

        let a = BigUint {
            value: [1, 2, 3, 4].vec(),
        };
        let b = BigUint {
            value: [5, 6, 7].vec(),
        };
        let result = &a * &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [5, 16, 34, 52, 45, 28].vec()
            },
        );
        let result = &b * &a;
        assert_eq!(
            &result,
            &BigUint {
                value: [5, 16, 34, 52, 45, 28].vec()
            },
        );

        let a = BigUint {
            value: [u64::MAX].vec(),
        };
        let b = BigUint {
            value: [u64::MAX].vec(),
        };
        let result = &a * &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [1, u64::MAX - 1].vec()
            },
        );
        let result = &b * &a;
        assert_eq!(
            &result,
            &BigUint {
                value: [1, u64::MAX - 1].vec()
            },
        );

        let a = BigUint {
            value: [u64::MAX, u64::MAX, u64::MAX].vec(),
        };
        let b = BigUint {
            value: [u64::MAX].vec(),
        };
        let result = &a * &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [1, u64::MAX, u64::MAX, u64::MAX - 1].vec()
            },
        );
        let result = &b * &a;
        assert_eq!(
            &result,
            &BigUint {
                value: [1, u64::MAX, u64::MAX, u64::MAX - 1].vec()
            },
        );
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    #[wasm_bindgen_test]
    fn test_div_by_zero() {
        let a = BigUint { value: [1].vec() };
        let result = &a / &BigUint::ZERO;
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    #[wasm_bindgen_test]
    fn test_div_zero_by_zero() {
        let result = &BigUint::ZERO / &BigUint::ZERO;
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_div_simple() {
        let a = BigUint { value: [2].vec() };
        let b = BigUint { value: [7].vec() };
        let result = &a / &b;
        assert_eq!(&result, &BigUint::ZERO);

        let a = BigUint { value: [7].vec() };
        let result = &a / &a;
        assert_eq!(&result, &BigUint { value: [1].vec() });

        let a = BigUint { value: [7].vec() };
        let b = BigUint { value: [2].vec() };
        let result = &a / &b;
        assert_eq!(&result, &BigUint { value: [3].vec() });

        let a = BigUint {
            value: [6, 8].vec(),
        };
        let b = BigUint { value: [2].vec() };
        let result = &a / &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [3, 4].vec()
            }
        );

        let a = BigUint {
            value: [4, 7].vec(),
        };
        let b = BigUint { value: [2].vec() };
        let result = &a / &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [(1 << 63) + 2, 3].vec()
            }
        );

        let a = BigUint {
            value: [0, 4].vec(),
        };
        let b = BigUint {
            value: [1, 2].vec(),
        };
        let result = &a / &b;
        assert_eq!(&result, &BigUint { value: [1].vec() });

        let a = BigUint {
            value: [1, 1].vec(),
        };
        let b = BigUint { value: [1].vec() };
        let result = &a / &b;
        assert_eq!(
            &result,
            &BigUint {
                value: [1, 1].vec()
            }
        );
    }
}

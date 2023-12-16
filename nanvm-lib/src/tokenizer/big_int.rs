use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Neg, Sub},
};

use crate::common::array::ArrayEx;
use crate::tokenizer::big_uint::BigUint;

#[derive(Debug, PartialEq, Clone, Eq, Default)]
pub struct BigInt {
    pub sign: Sign,
    pub value: BigUint,
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum Sign {
    Positive = 1,
    Negative = -1,
}

impl Default for Sign {
    fn default() -> Self {
        Self::Positive
    }
}

impl BigInt {
    pub const ZERO: BigInt = BigInt {
        sign: Sign::Positive,
        value: BigUint::ZERO,
    };

    fn normalize(&mut self) {
        self.value.normalize();
        if self.value.is_zero() {
            self.sign = Sign::Positive;
        }
    }

    pub fn is_zero(&self) -> bool {
        self.value.is_zero()
    }

    pub fn new(sign: Sign, value: BigUint) -> BigInt {
        BigInt { sign, value }
    }

    pub fn from_u64(n: u64) -> Self {
        BigInt {
            sign: Sign::Positive,
            value: BigUint { value: [n].vec() },
        }
    }

    pub fn abs(self) -> BigInt {
        match self.sign {
            Sign::Positive => self,
            Sign::Negative => BigInt { sign: Sign::Positive, value: self.value }
        }
    }

    pub fn from_i64(n: i64) -> Self {
        let sign = if n < 0 {
            Sign::Negative
        } else {
            Sign::Positive
        };
        BigInt {
            sign,
            value: BigUint {
                value: [(n * sign as i64) as u64].vec(),
            },
        }
    }
}

impl Add for &BigInt {
    type Output = BigInt;

    fn add(self, other: Self) -> Self::Output {
        match self.sign == other.sign {
            true => BigInt {
                sign: self.sign,
                value: &self.value + &other.value,
            },
            false => match self.value.cmp(&other.value) {
                Ordering::Equal => BigInt::ZERO,
                Ordering::Greater => BigInt {
                    sign: self.sign,
                    value: &self.value - &other.value,
                },
                Ordering::Less => BigInt {
                    sign: other.sign,
                    value: &other.value - &self.value,
                },
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
        if self.value.is_zero() {
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

        self.value.cmp(&other.value)
    }
}

impl Mul for &BigInt {
    type Output = BigInt;

    fn mul(self, other: Self) -> Self::Output {
        if self.is_zero() || other.is_zero() {
            return BigInt::ZERO;
        }
        let value = &self.value * &other.value;
        let sign = match self.sign == other.sign {
            true => Sign::Positive,
            false => Sign::Negative,
        };
        let mut result = BigInt { sign, value };
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

        let sign = match self.sign == d.sign {
            true => Sign::Positive,
            false => Sign::Negative,
        };

        let value = &self.value / &d.value;
        BigInt { sign, value }
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{common::array::ArrayEx, tokenizer::big_uint::BigUint};

    use super::{BigInt, Sign};

    #[test]
    #[wasm_bindgen_test]
    fn test_ord() {
        let a = BigInt::from_u64(1);
        let b = BigInt::from_u64(1);
        assert_eq!(a.cmp(&b), Ordering::Equal);

        let a = BigInt::from_u64(1);
        let b = BigInt::from_u64(2);
        assert_eq!(a.cmp(&b), Ordering::Less);

        let a = BigInt::from_u64(1);
        let b = BigInt::from_i64(-2);
        assert_eq!(a.cmp(&b), Ordering::Greater);

        let a = BigInt {
            sign: Sign::Positive,
            value: BigUint {
                value: [1, 2].vec(),
            },
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: BigUint {
                value: [2, 1].vec(),
            },
        };
        assert_eq!(a.cmp(&b), Ordering::Greater);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_add_same_sign() {
        let a = BigInt::from_u64(1);
        let b = BigInt::from_u64(2);
        let result = &a + &b;
        assert_eq!(&result, &BigInt::from_u64(3));

        let a = BigInt::from_i64(-1);
        let b = BigInt::from_i64(-2);
        let result = &a + &b;
        assert_eq!(&result, &BigInt::from_i64(-3));

        let a = BigInt::from_u64(1);
        let b = BigInt {
            sign: Sign::Positive,
            value: BigUint {
                value: [2, 4].vec(),
            },
        };
        let result = &a + &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: BigUint {
                    value: [3, 4].vec()
                }
            }
        );

        let a = BigInt::from_u64(1 << 63);
        let b = BigInt::from_u64(1 << 63);
        let result = &a + &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: BigUint {
                    value: [0, 1].vec()
                }
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_add_different_sign() {
        let a = BigInt::from_u64(1 << 63);
        let b = BigInt {
            sign: Sign::Negative,
            value: BigUint {
                value: [1 << 63].vec(),
            },
        };
        let result = &a + &b;
        assert_eq!(&result, &BigInt::ZERO);

        let a = BigInt::from_u64(3);
        let b = BigInt::from_i64(-2);
        let result = &a + &b;
        assert_eq!(&result, &BigInt::from_u64(1));
        let result = &b + &a;
        assert_eq!(&result, &BigInt::from_u64(1));

        let a = BigInt {
            sign: Sign::Positive,
            value: BigUint {
                value: [0, 1].vec(),
            },
        };
        let b = BigInt::from_i64(-1);
        let result = &a + &b;
        assert_eq!(&result, &BigInt::from_u64(u64::MAX));
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_add_overflow() {
        let a = BigInt {
            sign: Sign::Positive,
            value: BigUint {
                value: [u64::MAX, 0, 1].vec(),
            },
        };
        let b = BigInt {
            sign: Sign::Positive,
            value: BigUint {
                value: [u64::MAX, u64::MAX].vec(),
            },
        };
        let result = &a + &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: BigUint {
                    value: [u64::MAX - 1, 0, 2].vec()
                }
            }
        );
        let result = &b + &a;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: BigUint {
                    value: [u64::MAX - 1, 0, 2].vec()
                }
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_sub_same_sign() {
        let a = BigInt::from_u64(1 << 63);
        let b = BigInt::from_u64(1 << 63);
        let result = a - b;
        assert_eq!(&result, &BigInt::ZERO);

        let a = BigInt::from_u64(3);
        let b = BigInt::from_u64(2);
        let result = a.clone() - b.clone();
        assert_eq!(&result, &BigInt::from_u64(1));
        let result = b - a;
        assert_eq!(&result, &BigInt::from_i64(-1));

        let a = BigInt {
            sign: Sign::Positive,
            value: BigUint {
                value: [0, 1].vec(),
            },
        };
        let b = BigInt::from_u64(1);
        let result = a - b;
        assert_eq!(&result, &BigInt::from_u64(u64::MAX));
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_sub_different_sign() {
        let a = BigInt::from_u64(1);
        let b = BigInt::from_i64(-2);
        let result = a - b;
        assert_eq!(&result, &BigInt::from_u64(3));

        let a = BigInt::from_i64(-1);
        let b = BigInt::from_u64(2);
        let result = a - b;
        assert_eq!(&result, &BigInt::from_i64(-3));

        let a = BigInt::from_u64(1);
        let b = BigInt {
            sign: Sign::Negative,
            value: BigUint {
                value: [2, 4].vec(),
            },
        };
        let result = a - b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: BigUint {
                    value: [3, 4].vec()
                }
            }
        );

        let a = BigInt::from_u64(1 << 63);
        let b = BigInt {
            sign: Sign::Negative,
            value: BigUint {
                value: [1 << 63].vec(),
            },
        };
        let result = a - b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: BigUint {
                    value: [0, 1].vec()
                }
            }
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_mul() {
        let a = BigInt::from_u64(1);
        let result = &a * &BigInt::ZERO;
        assert_eq!(&result, &BigInt::ZERO);
        let result = &BigInt::ZERO * &a;
        assert_eq!(&result, &BigInt::ZERO);

        let a = BigInt::from_i64(-1);
        let result = &a * &BigInt::ZERO;
        assert_eq!(&result, &BigInt::ZERO);
        let result = &BigInt::ZERO * &a;
        assert_eq!(&result, &BigInt::ZERO);

        let a = BigInt::from_i64(-1);
        let b = BigInt::from_i64(-1);
        let result = &a * &b;
        assert_eq!(&result, &BigInt::from_u64(1));

        let a = BigInt {
            sign: Sign::Negative,
            value: BigUint {
                value: [1, 2, 3, 4].vec(),
            },
        };
        let b = BigInt {
            sign: Sign::Negative,
            value: BigUint {
                value: [5, 6, 7].vec(),
            },
        };
        let result = &a * &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: BigUint {
                    value: [5, 16, 34, 52, 45, 28].vec()
                }
            },
        );
        let result = &b * &a;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: BigUint {
                    value: [5, 16, 34, 52, 45, 28].vec()
                }
            },
        );

        let a = BigInt {
            sign: Sign::Negative,
            value: BigUint {
                value: [u64::MAX].vec(),
            },
        };
        let b = BigInt {
            sign: Sign::Negative,
            value: BigUint {
                value: [u64::MAX].vec(),
            },
        };
        let result = &a * &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: BigUint {
                    value: [1, u64::MAX - 1].vec()
                }
            },
        );
        let result = &b * &a;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: BigUint {
                    value: [1, u64::MAX - 1].vec()
                }
            },
        );

        let a = BigInt {
            sign: Sign::Negative,
            value: BigUint {
                value: [u64::MAX, u64::MAX, u64::MAX].vec(),
            },
        };
        let b = BigInt {
            sign: Sign::Negative,
            value: BigUint {
                value: [u64::MAX].vec(),
            },
        };
        let result = &a * &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: BigUint {
                    value: [1, u64::MAX, u64::MAX, u64::MAX - 1].vec()
                }
            },
        );
        let result = &b * &a;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: BigUint {
                    value: [1, u64::MAX, u64::MAX, u64::MAX - 1].vec()
                }
            },
        );
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    #[wasm_bindgen_test]
    fn test_div_by_zero() {
        let a = BigInt::from_u64(1);
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
        let a = BigInt::from_u64(2);
        let b = BigInt::from_u64(7);
        let result = &a / &b;
        assert_eq!(&result, &BigInt::ZERO);

        let a = BigInt::from_u64(7);
        let result = &a / &a;
        assert_eq!(&result, &BigInt::from_u64(1));

        let a = BigInt::from_u64(7);
        let b = BigInt::from_u64(2);
        let result = &a / &b;
        assert_eq!(&result, &BigInt::from_u64(3));

        let a = BigInt {
            sign: Sign::Positive,
            value: BigUint {
                value: [6, 8].vec(),
            },
        };
        let b = BigInt::from_u64(2);
        let result = &a / &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: BigUint {
                    value: [3, 4].vec()
                }
            }
        );

        let a = BigInt {
            sign: Sign::Positive,
            value: BigUint {
                value: [4, 7].vec(),
            },
        };
        let b = BigInt::from_u64(2);
        let result = &a / &b;
        assert_eq!(
            &result,
            &BigInt {
                sign: Sign::Positive,
                value: BigUint {
                    value: [(1 << 63) + 2, 3].vec()
                }
            }
        );
    }
}

use std::{iter, ops::Add};

use crate::common::default::default;

#[derive(Debug, PartialEq)]
struct BigInt {
    sign: Sign,
    value: Vec<u64>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Sign {
    Positive,
    Negative,
}

impl Add for BigInt {
    type Output = BigInt;

    fn add(self, rhs: Self) -> Self::Output {
        match (self.sign, rhs.sign) {
            (Sign::Positive, Sign::Positive) => add_same_sign(self, rhs),
            (Sign::Negative, Sign::Negative) => add_same_sign(self, rhs),
            _ => todo!(),
        }
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

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::common::array::ArrayEx;

    use super::{BigInt, Sign};

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

use std::{cmp::Ordering, iter};

use crate::{
    common::{bit_subset64::BitSubset64, default::default},
    mem::{
        block::Block,
        flexible_array::{
            constructor::FlexibleArrayConstructor, header::FlexibleArrayHeader, FlexibleArray,
        },
        manager::{Dealloc, Manager},
        mut_ref::MutRef,
        ref_::Ref,
    },
};

use super::{bitset::BIGINT, ref_cast::RefCast};

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum Sign {
    Positive = 1,
    Negative = -1,
}

pub struct JsBigintHeader {
    len: isize,
}

pub type JsBigint = FlexibleArray<u64, JsBigintHeader>;

pub type JsBigintRef<D> = Ref<JsBigint, D>;

pub type JsBigintMutRef<D> = MutRef<JsBigint, D>;

impl FlexibleArrayHeader for JsBigintHeader {
    fn len(&self) -> usize {
        self.len.unsigned_abs()
    }
}

impl<D: Dealloc> RefCast<D> for JsBigint {
    const REF_SUBSET: BitSubset64<*const Block<JsBigint, D>> = BIGINT.cast();
}

pub fn new_bigint<M: Manager, I: ExactSizeIterator<Item = u64>>(
    m: M,
    sign: Sign,
    i: impl IntoIterator<IntoIter = I>,
) -> JsBigintMutRef<M::Dealloc> {
    let items = i.into_iter();
    m.new(FlexibleArrayConstructor::new(
        JsBigintHeader {
            len: (items.len() as isize) * sign as isize,
        },
        items,
    ))
}

pub fn zero<M: Manager>(m: M) -> JsBigintMutRef<M::Dealloc> {
    new_bigint(m, Sign::Positive, iter::empty())
}

pub fn from_u64<M: Manager>(m: M, sign: Sign, n: u64) -> JsBigintMutRef<M::Dealloc> {
    if n == 0 {
        return zero(m);
    }
    new_bigint(m, sign, iter::once(n))
}

pub fn add<M: Manager>(m: M, lhs: &JsBigint, rhs: &JsBigint) -> JsBigintMutRef<M::Dealloc> {
    if lhs.sign() == rhs.sign() {
        new_bigint(m, lhs.sign(), add_vec(lhs.items(), rhs.items()))
    } else {
        match cmp_vec(lhs.items(), rhs.items()) {
            Ordering::Equal => zero(m),
            Ordering::Greater => new_bigint(m, lhs.sign(), sub_vec(lhs.items(), rhs.items())),
            Ordering::Less => new_bigint(m, rhs.sign(), sub_vec(rhs.items(), lhs.items())),
        }
    }
}

pub fn is_zero(value: &JsBigint) -> bool {
    value.items().is_empty()
}

pub fn negative<M: Manager>(m: M, value: &JsBigint) -> JsBigintMutRef<M::Dealloc> {
    if is_zero(value) {
        return zero(m);
    }
    let new_sign = match value.sign() {
        Sign::Positive => Sign::Negative,
        Sign::Negative => Sign::Positive,
    };
    new_bigint(m, new_sign, value.items().iter().copied())
}

pub fn sub<M: Manager>(_m: M, _lhs: &JsBigint, _rhs: &JsBigint) -> JsBigintMutRef<M::Dealloc> {
    todo!()
}

impl JsBigint {
    fn sign(&self) -> Sign {
        if self.header.len < 0 {
            Sign::Negative
        } else {
            Sign::Positive
        }
    }
}

fn add_vec(lhs: &[u64], rhs: &[u64]) -> Vec<u64> {
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
    value
}

fn sub_vec(lhs: &[u64], rhs: &[u64]) -> Vec<u64> {
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
    let res = value;
    normalize_vec(res)
}

fn normalize_vec(mut vec: Vec<u64>) -> Vec<u64> {
    while let Some(&0) = vec.last() {
        vec.pop();
    }
    vec
}

fn cmp_vec(lhs: &[u64], rhs: &[u64]) -> Ordering {
    let self_len = lhs.len();
    let other_len: usize = rhs.len();
    if self_len != other_len {
        return self_len.cmp(&other_len);
    }
    for (self_digit, other_digit) in lhs.iter().copied().rev().zip(rhs.iter().copied().rev()) {
        if self_digit != other_digit {
            return self_digit.cmp(&other_digit);
        }
    }
    Ordering::Equal
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        js::{
            any::Any,
            js_bigint::{JsBigintRef, Sign},
            type_::Type,
        },
        mem::global::Global,
    };

    use super::{add, from_u64};

    #[test]
    #[wasm_bindgen_test]
    fn test_add_u64() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = from_u64(Global(), Sign::Positive, 2);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let sum: BigintRef = add(Global(), a, b).to_ref();
        let u = A::move_from(sum);
        assert_eq!(u.get_type(), Type::Bigint);
        {
            let o = u.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[3]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, 1);
        let b_ref = from_u64(Global(), Sign::Negative, 2);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let sum: BigintRef = add(Global(), a, b).to_ref();
        let u = A::move_from(sum);
        assert_eq!(u.get_type(), Type::Bigint);
        {
            let o = u.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[3]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = from_u64(Global(), Sign::Negative, 2);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let sum: BigintRef = add(Global(), a, b).to_ref();
        let u = A::move_from(sum);
        assert_eq!(u.get_type(), Type::Bigint);
        {
            let o = u.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, 100);
        let b_ref = from_u64(Global(), Sign::Positive, 100);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let sum: BigintRef = add(Global(), a, b).to_ref();
        let u = A::move_from(sum);
        assert_eq!(u.get_type(), Type::Bigint);
        {
            let o = u.try_move::<BigintRef>().unwrap();
            assert!(o.items().is_empty());
        }

        let a_ref = from_u64(Global(), Sign::Positive, 1 << 63);
        let b_ref = from_u64(Global(), Sign::Positive, 1 << 63);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let sum: BigintRef = add(Global(), a, b).to_ref();
        let u = A::move_from(sum);
        assert_eq!(u.get_type(), Type::Bigint);
        {
            let o = u.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[0, 1]);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_add_overflow() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = from_u64(Global(), Sign::Positive, 1 << 63);
        let b_ref = from_u64(Global(), Sign::Positive, 1 << 63);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let sum: BigintRef = add(Global(), a, b).to_ref();
        let u = A::move_from(sum);
        assert_eq!(u.get_type(), Type::Bigint);
        {
            let o = u.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[0, 1]);
        }
    }
}

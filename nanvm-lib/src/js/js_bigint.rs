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

impl Sign {
    fn opposite(self) -> Self {
        match self {
            Self::Positive => Sign::Negative,
            Self::Negative => Sign::Positive,
        }
    }
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
    new_bigint(m, value.sign().opposite(), value.items().iter().copied())
}

pub fn sub<M: Manager>(m: M, lhs: &JsBigint, rhs: &JsBigint) -> JsBigintMutRef<M::Dealloc> {
    if lhs.sign() != rhs.sign() {
        new_bigint(m, lhs.sign(), add_vec(lhs.items(), rhs.items()))
    } else {
        match cmp_vec(lhs.items(), rhs.items()) {
            Ordering::Equal => zero(m),
            Ordering::Greater => new_bigint(m, lhs.sign(), sub_vec(lhs.items(), rhs.items())),
            Ordering::Less => {
                new_bigint(m, rhs.sign().opposite(), sub_vec(rhs.items(), lhs.items()))
            }
        }
    }
}

pub fn shl<M: Manager>(m: M, lhs: &JsBigint, rhs: &JsBigint) -> JsBigintMutRef<M::Dealloc> {
    if is_zero(lhs) {
        return zero(m);
    }

    if is_zero(rhs) {
        return new_bigint(m, lhs.sign(), lhs.items().to_vec());
    }

    if rhs.sign() == Sign::Negative {
        panic!("Shift right operand should be positive")
    }

    if rhs.items().len() != 1 {
        panic!("Maximum bigint size exceeded")
    }

    let mut vec = lhs.items().to_vec();
    let shift_mod = rhs.items()[0] & ((1 << 6) - 1);
    if shift_mod > 0 {
        let len = vec.len();
        vec.push(0);
        for i in (0..=len - 1).rev() {
            let mut digit = vec[i] as u128;
            digit <<= shift_mod;
            vec[i + 1] |= (digit >> 64) as u64;
            vec[i] = digit as u64;
        }
    }

    let number_of_zeros = (rhs.items()[0] / 64) as usize;
    if number_of_zeros > 0 {
        let mut zeros_vector: Vec<_> = vec![0; number_of_zeros];
        zeros_vector.extend(vec);
        vec = zeros_vector;
    }

    vec = normalize_vec(vec);
    new_bigint(m, lhs.sign(), vec)
}

pub fn shr<M: Manager>(m: M, lhs: &JsBigint, rhs: &JsBigint) -> JsBigintMutRef<M::Dealloc> {
    if is_zero(lhs) {
        return zero(m);
    }

    if is_zero(rhs) {
        return new_bigint(m, lhs.sign(), lhs.items().to_vec());
    }

    if rhs.sign() == Sign::Negative {
        panic!("Shift right operand should be positive")
    }

    let number_to_remove = (rhs.items()[0] / 64) as usize;
    if number_to_remove >= lhs.items().len() {
        return match lhs.sign() {
            Sign::Positive => zero(m),
            Sign::Negative => from_u64(m, Sign::Negative, 1),
        };
    }

    let mut vec = lhs.items().to_vec();
    vec = vec.split_off(number_to_remove);
    let shift_mod = rhs.items()[0] & ((1 << 6) - 1);
    if shift_mod > 0 {
        let len = vec.len();
        let mask = 1 << (shift_mod - 1);
        let mut i = 0;
        loop {
            vec[i] >>= shift_mod;
            i += 1;
            if i == len {
                break;
            }
            vec[i - 1] |= (vec[i] & mask) << (64 - shift_mod);
        }
    }

    vec = normalize_vec(vec);
    if vec.is_empty() && lhs.sign() == Sign::Negative {
        return from_u64(m, Sign::Negative, 1);
    }
    new_bigint(m, lhs.sign(), vec)
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
            js_bigint::{new_bigint, shl, shr, sub, zero, JsBigintRef, Sign},
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
        let res = A::move_from(sum);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[3]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, 1);
        let b_ref = from_u64(Global(), Sign::Negative, 2);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let sum: BigintRef = add(Global(), a, b).to_ref();
        let res = A::move_from(sum);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[3]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = from_u64(Global(), Sign::Negative, 2);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let sum: BigintRef = add(Global(), a, b).to_ref();
        let res = A::move_from(sum);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
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
        let res = A::move_from(sum);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
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
        let res = A::move_from(sum);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[0, 1]);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_sub_u64() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = from_u64(Global(), Sign::Positive, 2);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let sum: BigintRef = sub(Global(), a, b).to_ref();
        let res = A::move_from(sum);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = from_u64(Global(), Sign::Negative, 2);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let sum: BigintRef = sub(Global(), a, b).to_ref();
        let res = A::move_from(sum);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[3]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, 100);
        let b_ref = from_u64(Global(), Sign::Negative, 100);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let sum: BigintRef = sub(Global(), a, b).to_ref();
        let res = A::move_from(sum);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert!(o.items().is_empty());
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_shl_zero() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = zero(Global());
        let b_ref = from_u64(Global(), Sign::Positive, 2);
        let a = a_ref.deref();
        let b = b_ref.deref();

        let c: BigintRef = shl(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert!(o.items().is_empty());
        }

        let c: BigintRef = shl(Global(), b, a).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[2]);
        }

        let c: BigintRef = shl(Global(), a, a).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert!(o.items().is_empty());
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_shl() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = from_u64(Global(), Sign::Positive, 1);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = shl(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[2]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, 1);
        let b_ref = from_u64(Global(), Sign::Positive, 1);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = shl(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[2]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 5);
        let b_ref = from_u64(Global(), Sign::Positive, 63);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = shl(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1 << 63, 2]);
        }

        let a_ref = new_bigint(Global(), Sign::Positive, [5, 9]);
        let b_ref = from_u64(Global(), Sign::Positive, 63);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = shl(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1 << 63, (1 << 63) + 2, 4]);
        }

        let a_ref = new_bigint(Global(), Sign::Positive, [5, 9]);
        let b_ref = from_u64(Global(), Sign::Positive, 64);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = shl(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[0, 5, 9]);
        }

        let a_ref = new_bigint(Global(), Sign::Positive, [5, 9]);
        let b_ref = from_u64(Global(), Sign::Positive, 65);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = shl(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[0, 10, 18]);
        }
    }

    #[test]
    #[should_panic(expected = "Maximum bigint size exceeded")]
    #[wasm_bindgen_test]
    fn test_shl_overflow() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = new_bigint(Global(), Sign::Positive, [1, 1]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let _c: BigintRef = shl(Global(), a, b).to_ref();
    }

    #[test]
    #[should_panic(expected = "Shift right operand should be positive")]
    #[wasm_bindgen_test]
    fn test_shl_negative_rhs() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = from_u64(Global(), Sign::Negative, 1);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let _c: BigintRef = shl(Global(), a, b).to_ref();
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_shr_zero() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = zero(Global());
        let b_ref = from_u64(Global(), Sign::Positive, 2);
        let a = a_ref.deref();
        let b = b_ref.deref();

        let c: BigintRef = shr(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert!(o.items().is_empty());
        }

        let c: BigintRef = shr(Global(), b, a).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[2]);
        }

        let c: BigintRef = shr(Global(), a, a).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert!(o.items().is_empty());
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_shr() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = new_bigint(Global(), Sign::Positive, [1, 1, 1, 1]);
        let b_ref = from_u64(Global(), Sign::Positive, 256);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = shr(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert!(o.items().is_empty());
        }

        let a_ref = new_bigint(Global(), Sign::Negative, [1, 1, 1, 1]);
        let b_ref = from_u64(Global(), Sign::Positive, 256);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = shr(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = from_u64(Global(), Sign::Positive, 1);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = shr(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert!(o.items().is_empty());
        }

        let a_ref = from_u64(Global(), Sign::Negative, 1);
        let b_ref = from_u64(Global(), Sign::Positive, 1);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = shr(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 2);
        let b_ref = from_u64(Global(), Sign::Positive, 1);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = shr(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = new_bigint(Global(), Sign::Positive, [1, 5, 9]);
        let b_ref = from_u64(Global(), Sign::Positive, 65);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = shr(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[(1 << 63) + 2, 4]);
        }
    }

    #[test]
    #[should_panic(expected = "Shift right operand should be positive")]
    #[wasm_bindgen_test]
    fn test_shr_negative_rhs() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = from_u64(Global(), Sign::Negative, 1);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let _c: BigintRef = shr(Global(), a, b).to_ref();
    }
}
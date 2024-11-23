use std::{cmp::Ordering, iter};

use crate::{
    common::{bit_subset64::BitSubset64, default::default, vec::new_resize},
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

#[derive(Debug)]
pub struct JsBigintHeader {
    len: isize,
}

pub struct TwosComplement {
    sign: Sign,
    vec: Vec<u64>,
}

impl JsBigint {
    pub fn header_len(&self) -> isize {
        self.header.len
    }
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

impl TwosComplement {
    fn repeat(&self) -> u64 {
        match self.sign {
            Sign::Positive => 0,
            Sign::Negative => u64::MAX,
        }
    }
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

pub fn not<M: Manager>(m: M, value: &JsBigint) -> JsBigintMutRef<M::Dealloc> {
    if is_zero(value) {
        return from_u64(m, Sign::Negative, 1);
    }
    match value.sign() {
        Sign::Positive => new_bigint(m, Sign::Negative, add_vec(value.items(), &[1])),
        Sign::Negative => new_bigint(
            m,
            Sign::Positive,
            normalize_vec(sub_vec(value.items(), &[1])),
        ),
    }
}

pub fn add<M: Manager>(m: M, lhs: &JsBigint, rhs: &JsBigint) -> JsBigintMutRef<M::Dealloc> {
    if lhs.sign() == rhs.sign() {
        new_bigint(m, lhs.sign(), add_vec(lhs.items(), rhs.items()))
    } else {
        match cmp_vec(lhs.items(), rhs.items()) {
            Ordering::Equal => zero(m),
            Ordering::Greater => new_bigint(m, lhs.sign(), sub_vec_norm(lhs.items(), rhs.items())),
            Ordering::Less => new_bigint(m, rhs.sign(), sub_vec_norm(rhs.items(), lhs.items())),
        }
    }
}

pub fn is_zero(value: &JsBigint) -> bool {
    value.items().is_empty()
}

impl JsBigint {
    pub fn cmp(&self, other: &Self) -> Ordering {
        match self.header_len().cmp(&other.header_len()) {
            Ordering::Equal => cmp_vec(self.items(), other.items()),
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        }
    }
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
            Ordering::Greater => new_bigint(m, lhs.sign(), sub_vec_norm(lhs.items(), rhs.items())),
            Ordering::Less => new_bigint(
                m,
                rhs.sign().opposite(),
                sub_vec_norm(rhs.items(), lhs.items()),
            ),
        }
    }
}

pub fn mul<M: Manager>(m: M, lhs: &JsBigint, rhs: &JsBigint) -> JsBigintMutRef<M::Dealloc> {
    if is_zero(lhs) || is_zero(rhs) {
        return zero(m);
    }
    let sign = match lhs.sign() == rhs.sign() {
        true => Sign::Positive,
        false => Sign::Negative,
    };
    new_bigint(m, sign, mul_vec(lhs.items(), rhs.items()))
}

fn mul_vec(lhs: &[u64], rhs: &[u64]) -> Vec<u64> {
    let lhs_max = lhs.len() - 1;
    let rhs_max = rhs.len() - 1;
    let total_max = lhs_max + rhs_max + 1;
    let mut vec = new_resize(total_max + 1);
    let mut i: usize = 0;
    while i < total_max {
        let mut j = if i > rhs_max { i - rhs_max } else { 0 };
        let max = if i < lhs_max { i } else { lhs_max };
        while j <= max {
            vec = add_to_vec(vec, i, lhs[j] as u128 * rhs[i - j] as u128);
            j += 1;
        }
        i += 1;
    }
    normalize_vec(vec)
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

pub fn div_mod<M: Manager>(
    m: M,
    lhs: &JsBigint,
    rhs: &JsBigint,
) -> (JsBigintMutRef<M::Dealloc>, JsBigintMutRef<M::Dealloc>) {
    if is_zero(rhs) {
        panic!("attempt to divide by zero");
    }

    let sign = match lhs.sign() == rhs.sign() {
        true => Sign::Positive,
        false => Sign::Negative,
    };

    match cmp_vec(lhs.items(), rhs.items()) {
        Ordering::Less => (zero(m), new_bigint(m, sign, rhs.items().to_vec())),
        Ordering::Equal => (from_u64(m, sign, 1), zero(m)),
        Ordering::Greater => {
            let mut a = lhs.items().to_vec();
            let b = rhs.items();
            let mut result: Vec<u64> = default();
            loop {
                if cmp_vec(&a, b) == Ordering::Less {
                    return (new_bigint(m, sign.clone(), result), new_bigint(m, sign, a));
                }
                let a_high_digit = a.len() - 1;
                let b_high_digit = b.len() - 1;
                let a_high = a[a_high_digit];
                let b_high = b[b_high_digit];
                let (q_index, q_digit) = match b_high.cmp(&a_high) {
                    Ordering::Less | Ordering::Equal => {
                        (a_high_digit - b_high_digit, a_high / b_high)
                    }
                    Ordering::Greater => {
                        let a_high_2 = ((a_high as u128) << 64) + a[a_high_digit - 1] as u128;
                        (
                            a_high_digit - b_high_digit - 1,
                            (a_high_2 / b_high as u128) as u64,
                        )
                    }
                };
                let mut q = new_resize(q_index + 1);
                q[q_index] = q_digit;
                let mut m = mul_vec(b, &q);
                if a.cmp(&m) == Ordering::Less {
                    q[q_index] = q_digit - 1;
                    m = mul_vec(b, &q);
                }
                a = sub_vec(&a, &m);
                a = normalize_vec(a);
                result = add_vec(&result, &q);
            }
        }
    }
}

pub fn pow<M: Manager>(m: M, lhs: &JsBigint, rhs: &JsBigint) -> JsBigintMutRef<M::Dealloc> {
    if rhs.sign() == Sign::Negative {
        panic!("exponent must be positive");
    }

    if lhs.items().len() == 1 && lhs.items()[0] == 1 {
        return from_u64(m, Sign::Positive, 1);
    }

    if is_zero(lhs) {
        return if is_zero(rhs) {
            from_u64(m, Sign::Positive, 1)
        } else {
            zero(m)
        };
    }

    if is_zero(rhs) {
        return from_u64(m, Sign::Positive, 1);
    }

    if rhs.items().len() != 1 {
        panic!("maximum bigint size exceeded")
    }

    pow_u64(m, lhs, rhs.items()[0])
}

pub fn pow_u64<M: Manager>(m: M, lhs: &JsBigint, mut rhs: u64) -> JsBigintMutRef<M::Dealloc> {
    let mut res = [1].to_vec();
    let mut b = lhs.items().to_vec();
    let sign = if lhs.sign() == Sign::Positive || rhs & 1 == 0 {
        Sign::Positive
    } else {
        Sign::Negative
    };
    loop {
        if rhs == 0 {
            return new_bigint(m, sign, res);
        }
        if rhs & 1 > 0 {
            res = mul_vec(&res, &b);
        }
        rhs >>= 1;
        b = mul_vec(&b, &b);
    }
}

pub fn and<M: Manager>(m: M, lhs: &JsBigint, rhs: &JsBigint) -> JsBigintMutRef<M::Dealloc> {
    let lhs_tc = to_twos_complement(lhs);
    let rhs_tc = to_twos_complement(rhs);
    let res_tc = and_twos_complement(lhs_tc, rhs_tc);
    from_twos_complement(m, res_tc)
}

pub fn or<M: Manager>(m: M, lhs: &JsBigint, rhs: &JsBigint) -> JsBigintMutRef<M::Dealloc> {
    let lhs_tc = to_twos_complement(lhs);
    let rhs_tc = to_twos_complement(rhs);
    let res_tc = or_twos_complement(lhs_tc, rhs_tc);
    from_twos_complement(m, res_tc)
}

pub fn xor<M: Manager>(m: M, lhs: &JsBigint, rhs: &JsBigint) -> JsBigintMutRef<M::Dealloc> {
    let lhs_tc = to_twos_complement(lhs);
    let rhs_tc = to_twos_complement(rhs);
    let res_tc = xor_twos_complement(lhs_tc, rhs_tc);
    from_twos_complement(m, res_tc)
}

pub fn shl<M: Manager>(m: M, lhs: &JsBigint, rhs: &JsBigint) -> JsBigintMutRef<M::Dealloc> {
    if is_zero(lhs) {
        return zero(m);
    }

    if is_zero(rhs) {
        return new_bigint(m, lhs.sign(), lhs.items().to_vec());
    }

    if rhs.items().len() != 1 {
        return match rhs.sign() {
            Sign::Positive => panic!("Maximum bigint size exceeded"),
            Sign::Negative => shr_on_big(m, lhs.sign()),
        };
    }

    match rhs.sign() {
        Sign::Positive => shl_on_u64(m, lhs, rhs.items()[0]),
        Sign::Negative => shr_on_u64(m, lhs, rhs.items()[0]),
    }
}

pub fn shr<M: Manager>(m: M, lhs: &JsBigint, rhs: &JsBigint) -> JsBigintMutRef<M::Dealloc> {
    if is_zero(lhs) {
        return zero(m);
    }

    if is_zero(rhs) {
        return new_bigint(m, lhs.sign(), lhs.items().to_vec());
    }

    if rhs.items().len() != 1 {
        return match rhs.sign() {
            Sign::Positive => shr_on_big(m, lhs.sign()),
            Sign::Negative => panic!("Maximum bigint size exceeded"),
        };
    }

    match rhs.sign() {
        Sign::Positive => shr_on_u64(m, lhs, rhs.items()[0]),
        Sign::Negative => shl_on_u64(m, lhs, rhs.items()[0]),
    }
}

pub fn equals(lhs: &JsBigint, rhs: &JsBigint) -> bool {
    if lhs.sign() != rhs.sign() {
        return false;
    }
    return cmp_vec(lhs.items(), rhs.items()) == Ordering::Equal;
}

fn to_twos_complement(value: &JsBigint) -> TwosComplement {
    TwosComplement {
        sign: value.sign(),
        vec: match value.sign() {
            Sign::Positive => value.items().to_vec(),
            Sign::Negative => {
                let sub = sub_vec(value.items(), &[1]);
                let mut res: Vec<_> = default();
                for d in sub {
                    res.push(!d);
                }
                res
            }
        },
    }
}

fn from_twos_complement<M: Manager>(m: M, value: TwosComplement) -> JsBigintMutRef<M::Dealloc> {
    match value.sign {
        Sign::Positive => new_bigint(m, Sign::Positive, value.vec),
        Sign::Negative => {
            let sub = sub_vec(&value.vec, &[1]);
            let mut res: Vec<u64> = default();
            for d in sub {
                res.push(!d);
            }
            res = normalize_vec(res);
            if res.is_empty() {
                return from_u64(m, Sign::Negative, 1);
            }
            new_bigint(m, Sign::Negative, res)
        }
    }
}

fn and_twos_complement(lhs: TwosComplement, rhs: TwosComplement) -> TwosComplement {
    let sign = match lhs.sign == Sign::Negative && rhs.sign == Sign::Negative {
        true => Sign::Negative,
        false => Sign::Positive,
    };
    let mut vec: Vec<_> = default();
    for (a, b) in twos_complement_zip(&lhs, &rhs) {
        vec.push(a & b);
    }
    vec = normalize_vec(vec);
    TwosComplement { sign, vec }
}

fn or_twos_complement(lhs: TwosComplement, rhs: TwosComplement) -> TwosComplement {
    let sign = match lhs.sign == Sign::Negative || rhs.sign == Sign::Negative {
        true => Sign::Negative,
        false => Sign::Positive,
    };
    let mut vec: Vec<_> = default();
    for (a, b) in twos_complement_zip(&lhs, &rhs) {
        vec.push(a | b);
    }
    vec = normalize_vec(vec);
    TwosComplement { sign, vec }
}

fn xor_twos_complement(lhs: TwosComplement, rhs: TwosComplement) -> TwosComplement {
    let sign = match lhs.sign == rhs.sign {
        true => Sign::Positive,
        false => Sign::Negative,
    };
    let mut vec: Vec<_> = default();
    for (a, b) in twos_complement_zip(&lhs, &rhs) {
        vec.push(a ^ b);
    }
    vec = normalize_vec(vec);
    TwosComplement { sign, vec }
}

fn twos_complement_zip<'a>(
    lhs: &'a TwosComplement,
    rhs: &'a TwosComplement,
) -> impl Iterator<Item = (u64, u64)> + 'a {
    match rhs.vec.len() > lhs.vec.len() {
        true => rhs
            .vec
            .iter()
            .copied()
            .zip(lhs.vec.iter().copied().chain(iter::repeat(lhs.repeat()))),
        false => lhs
            .vec
            .iter()
            .copied()
            .zip(rhs.vec.iter().copied().chain(iter::repeat(rhs.repeat()))),
    }
}

impl JsBigint {
    pub fn sign(&self) -> Sign {
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

fn sub_vec_norm(lhs: &[u64], rhs: &[u64]) -> Vec<u64> {
    normalize_vec(sub_vec(lhs, rhs))
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
    value
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

fn shl_on_u64<M: Manager>(m: M, lhs: &JsBigint, rhs: u64) -> JsBigintMutRef<M::Dealloc> {
    let mut vec = lhs.items().to_vec();
    let shift_mod = rhs & ((1 << 6) - 1);
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

    let number_of_zeros = (rhs / 64) as usize;
    if number_of_zeros > 0 {
        let mut zeros_vector: Vec<_> = new_resize(number_of_zeros);
        zeros_vector.extend(vec);
        vec = zeros_vector;
    }

    vec = normalize_vec(vec);
    new_bigint(m, lhs.sign(), vec)
}

fn shr_on_u64<M: Manager>(m: M, lhs: &JsBigint, rhs: u64) -> JsBigintMutRef<M::Dealloc> {
    let number_to_remove = (rhs / 64) as usize;
    if number_to_remove >= lhs.items().len() {
        return shr_on_big(m, lhs.sign());
    }

    let mut vec = lhs.items().to_vec();
    vec = vec.split_off(number_to_remove);
    let shift_mod = rhs & ((1 << 6) - 1);
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

fn shr_on_big<M: Manager>(m: M, sign: Sign) -> JsBigintMutRef<M::Dealloc> {
    match sign {
        Sign::Positive => zero(m),
        Sign::Negative => from_u64(m, Sign::Negative, 1),
    }
}

#[cfg(test)]
mod test {
    use std::{cmp::Ordering, ops::Deref};

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        js::{any::Any, js_bigint::xor, type_::Type},
        mem::global::Global,
    };

    use super::{
        add, and, div_mod, from_u64, mul, new_bigint, not, or, pow, shl, shr, sub, zero,
        JsBigintRef, Sign,
    };

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
    fn test_cmp() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = from_u64(Global(), Sign::Positive, 1);
        let a = a_ref.deref();
        let b = b_ref.deref();
        assert_eq!(a.cmp(b), Ordering::Equal);

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = from_u64(Global(), Sign::Positive, 2);
        let a = a_ref.deref();
        let b = b_ref.deref();
        assert_eq!(a.cmp(b), Ordering::Less);

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = from_u64(Global(), Sign::Negative, 2);
        let a = a_ref.deref();
        let b = b_ref.deref();
        assert_eq!(a.cmp(b), Ordering::Greater);

        let a_ref = new_bigint(Global(), Sign::Positive, [1, 2]);
        let b_ref = new_bigint(Global(), Sign::Positive, [2, 1]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        assert_eq!(a.cmp(b), Ordering::Greater);
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

        let a_ref = from_u64(Global(), Sign::Positive, 2);
        let b_ref = from_u64(Global(), Sign::Negative, 1);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = shl(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1]);
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

        let a_ref = from_u64(Global(), Sign::Positive, 2);
        let b_ref = from_u64(Global(), Sign::Negative, 1);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = shr(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[4]);
        }
    }

    #[test]
    #[should_panic(expected = "Maximum bigint size exceeded")]
    #[wasm_bindgen_test]
    fn test_shr_overflow() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = new_bigint(Global(), Sign::Negative, [1, 1]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let _c: BigintRef = shr(Global(), a, b).to_ref();
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_and() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = new_bigint(Global(), Sign::Positive, [1, 3, 5, 7, 9]);
        let b_ref = new_bigint(Global(), Sign::Positive, [3, 5, 7, 9, 11]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = and(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.items(), &[1, 1, 5, 1, 9]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 12);
        let b_ref = from_u64(Global(), Sign::Negative, 9);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = and(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[4]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, 12);
        let b_ref = from_u64(Global(), Sign::Negative, 9);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = and(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[12]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = zero(Global());
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = and(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert!(o.items().is_empty());
        }

        let a_ref = from_u64(Global(), Sign::Negative, 1);
        let b_ref = zero(Global());
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = and(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert!(o.items().is_empty());
        }

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = new_bigint(Global(), Sign::Negative, [1, 1]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = and(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, 1);
        let b_ref = new_bigint(Global(), Sign::Positive, [1, 1]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = and(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1, 1]);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_or() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = new_bigint(Global(), Sign::Positive, [1, 3, 5, 7, 9]);
        let b_ref = new_bigint(Global(), Sign::Positive, [3, 5, 7, 9, 11]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = or(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.items(), &[3, 7, 7, 15, 11]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 12);
        let b_ref = from_u64(Global(), Sign::Negative, 9);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = or(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, 12);
        let b_ref = from_u64(Global(), Sign::Negative, 9);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = or(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[9]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = zero(Global());
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = or(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, 1);
        let b_ref = zero(Global());
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = or(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = new_bigint(Global(), Sign::Negative, [0, 1]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = or(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[u64::MAX]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, 1);
        let b_ref = new_bigint(Global(), Sign::Positive, [0, 1]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = or(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[1]);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_xor() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = new_bigint(Global(), Sign::Positive, [1, 3, 5, 7, 9]);
        let b_ref = new_bigint(Global(), Sign::Positive, [3, 5, 7, 9, 11]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = xor(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.items(), &[2, 6, 2, 14, 2]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 12);
        let b_ref = from_u64(Global(), Sign::Negative, 9);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = xor(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[5]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, 12);
        let b_ref = from_u64(Global(), Sign::Negative, 9);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = xor(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[3]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = zero(Global());
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = xor(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, 1);
        let b_ref = zero(Global());
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = xor(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = new_bigint(Global(), Sign::Negative, [0, 1]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = xor(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[u64::MAX]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, 1);
        let b_ref = new_bigint(Global(), Sign::Positive, [0, 1]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c: BigintRef = xor(Global(), a, b).to_ref();
        let res = A::move_from(c);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[1, 1]);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_not() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = zero(Global());
        let a = a_ref.deref();
        let not_a: BigintRef = not(Global(), a).to_ref();
        let res = A::move_from(not_a);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, 1);
        let a = a_ref.deref();
        let not_a: BigintRef = not(Global(), a).to_ref();
        let res = A::move_from(not_a);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert!(o.items().is_empty());
        }

        let a_ref = new_bigint(Global(), Sign::Positive, [1, 5, 9]);
        let a = a_ref.deref();
        let not_a: BigintRef = not(Global(), a).to_ref();
        let res = A::move_from(not_a);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[2, 5, 9]);
        }

        let a_ref = new_bigint(Global(), Sign::Negative, [1, 5, 9]);
        let a = a_ref.deref();
        let not_a: BigintRef = not(Global(), a).to_ref();
        let res = A::move_from(not_a);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[0, 5, 9]);
        }

        let a_ref = new_bigint(Global(), Sign::Negative, [0, 1]);
        let a = a_ref.deref();
        let not_a: BigintRef = not(Global(), a).to_ref();
        let res = A::move_from(not_a);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[u64::MAX]);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_mul() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = from_u64(Global(), Sign::Negative, 1);
        let zero_ref = zero(Global());
        let a = a_ref.deref();
        let b = b_ref.deref();
        let z = zero_ref.deref();

        let product: BigintRef = mul(Global(), a, z).to_ref();
        let res = A::move_from(product);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert!(o.items().is_empty());
        }

        let product: BigintRef = mul(Global(), z, a).to_ref();
        let res = A::move_from(product);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert!(o.items().is_empty());
        }

        let product: BigintRef = mul(Global(), b, z).to_ref();
        let res = A::move_from(product);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert!(o.items().is_empty());
        }

        let product: BigintRef = mul(Global(), z, b).to_ref();
        let res = A::move_from(product);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert!(o.items().is_empty());
        }

        let product: BigintRef = mul(Global(), a, b).to_ref();
        let res = A::move_from(product);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = new_bigint(Global(), Sign::Positive, [1, 2, 3, 4]);
        let b_ref = new_bigint(Global(), Sign::Positive, [5, 6, 7]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let product: BigintRef = mul(Global(), a, b).to_ref();
        let res = A::move_from(product);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[5, 16, 34, 52, 45, 28]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, u64::MAX);
        let b_ref = from_u64(Global(), Sign::Negative, u64::MAX);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let product: BigintRef = mul(Global(), a, b).to_ref();
        let res = A::move_from(product);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1, u64::MAX - 1]);
        }

        let a_ref = new_bigint(Global(), Sign::Negative, [u64::MAX, u64::MAX, u64::MAX]);
        let b_ref = from_u64(Global(), Sign::Negative, u64::MAX);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let product: BigintRef = mul(Global(), a, b).to_ref();
        let res = A::move_from(product);
        assert_eq!(res.get_type(), Type::Bigint);
        {
            let o = res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1, u64::MAX, u64::MAX, u64::MAX - 1]);
        }
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    #[wasm_bindgen_test]
    fn test_div_by_zero() {
        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = zero(Global());
        let a = a_ref.deref();
        let b = b_ref.deref();
        let _ = div_mod(Global(), a, b);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    #[wasm_bindgen_test]
    fn test_div_zero_by_zero() {
        let a_ref = zero(Global());
        let b_ref = zero(Global());
        let a = a_ref.deref();
        let b = b_ref.deref();
        let _ = div_mod(Global(), a, b);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_div() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = from_u64(Global(), Sign::Positive, 2);
        let b_ref = from_u64(Global(), Sign::Positive, 7);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let (d, m) = div_mod(Global(), a, b);
        let d_ref = d.to_ref();
        let d_res = A::move_from(d_ref);
        assert_eq!(d_res.get_type(), Type::Bigint);
        {
            let o = d_res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert!(o.items().is_empty());
        }
        let m_ref = m.to_ref();
        let m_res = A::move_from(m_ref);
        assert_eq!(m_res.get_type(), Type::Bigint);
        {
            let o = m_res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[7]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 7);
        let b_ref = from_u64(Global(), Sign::Positive, 2);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let (d, m) = div_mod(Global(), a, b);
        let d_ref = d.to_ref();
        let d_res = A::move_from(d_ref);
        assert_eq!(d_res.get_type(), Type::Bigint);
        {
            let o = d_res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[3]);
        }
        let m_ref = m.to_ref();
        let m_res = A::move_from(m_ref);
        assert_eq!(m_res.get_type(), Type::Bigint);
        {
            let o = m_res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = new_bigint(Global(), Sign::Negative, [1, 1]);
        let b_ref = from_u64(Global(), Sign::Negative, 1);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let (d, m) = div_mod(Global(), a, b);
        let d_ref = d.to_ref();
        let d_res = A::move_from(d_ref);
        assert_eq!(d_res.get_type(), Type::Bigint);
        {
            let o = d_res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1, 1]);
        }
        let m_ref = m.to_ref();
        let m_res = A::move_from(m_ref);
        assert_eq!(m_res.get_type(), Type::Bigint);
        {
            let o = m_res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert!(o.items().is_empty());
        }

        let a_ref = new_bigint(Global(), Sign::Positive, [7, 5]);
        let b_ref = new_bigint(Global(), Sign::Negative, [0, 3]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let (d, m) = div_mod(Global(), a, b);
        let d_ref = d.to_ref();
        let d_res = A::move_from(d_ref);
        assert_eq!(d_res.get_type(), Type::Bigint);
        {
            let o = d_res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[1]);
        }
        let m_ref = m.to_ref();
        let m_res = A::move_from(m_ref);
        assert_eq!(m_res.get_type(), Type::Bigint);
        {
            let o = m_res.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[7, 2]);
        }
    }

    #[test]
    #[should_panic(expected = "maximum bigint size exceeded")]
    #[wasm_bindgen_test]
    fn test_pow_overflow() {
        let a_ref = from_u64(Global(), Sign::Positive, 5);
        let b_ref = new_bigint(Global(), Sign::Positive, [100, 100]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let _ = pow(Global(), a, b);
    }

    #[test]
    #[should_panic(expected = "exponent must be positive")]
    #[wasm_bindgen_test]
    fn test_pow_negative() {
        let a_ref = from_u64(Global(), Sign::Positive, 5);
        let b_ref = new_bigint(Global(), Sign::Negative, [5]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let _ = pow(Global(), a, b);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_pow() {
        type A = Any<Global>;
        type BigintRef = JsBigintRef<Global>;

        let a_ref = from_u64(Global(), Sign::Positive, 100);
        let b_ref = zero(Global());
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c = pow(Global(), a, b).to_ref();
        let result = A::move_from(c);
        assert_eq!(result.get_type(), Type::Bigint);
        {
            let o = result.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 2);
        let b_ref = from_u64(Global(), Sign::Positive, 7);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c = pow(Global(), a, b).to_ref();
        let result = A::move_from(c);
        assert_eq!(result.get_type(), Type::Bigint);
        {
            let o = result.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[128]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, 2);
        let b_ref = from_u64(Global(), Sign::Positive, 7);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c = pow(Global(), a, b).to_ref();
        let result = A::move_from(c);
        assert_eq!(result.get_type(), Type::Bigint);
        {
            let o = result.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Negative);
            assert_eq!(o.items(), &[128]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 5);
        let b_ref = from_u64(Global(), Sign::Positive, 4);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c = pow(Global(), a, b).to_ref();
        let result = A::move_from(c);
        assert_eq!(result.get_type(), Type::Bigint);
        {
            let o = result.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[625]);
        }

        let a_ref = from_u64(Global(), Sign::Negative, 5);
        let b_ref = from_u64(Global(), Sign::Positive, 4);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c = pow(Global(), a, b).to_ref();
        let result = A::move_from(c);
        assert_eq!(result.get_type(), Type::Bigint);
        {
            let o = result.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[625]);
        }

        let a_ref = zero(Global());
        let b_ref = zero(Global());
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c = pow(Global(), a, b).to_ref();
        let result = A::move_from(c);
        assert_eq!(result.get_type(), Type::Bigint);
        {
            let o = result.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = zero(Global());
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c = pow(Global(), a, b).to_ref();
        let result = A::move_from(c);
        assert_eq!(result.get_type(), Type::Bigint);
        {
            let o = result.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1]);
        }

        let a_ref = from_u64(Global(), Sign::Positive, 1);
        let b_ref = new_bigint(Global(), Sign::Positive, [100, 100]);
        let a = a_ref.deref();
        let b = b_ref.deref();
        let c = pow(Global(), a, b).to_ref();
        let result = A::move_from(c);
        assert_eq!(result.get_type(), Type::Bigint);
        {
            let o = result.try_move::<BigintRef>().unwrap();
            assert_eq!(o.sign(), Sign::Positive);
            assert_eq!(o.items(), &[1]);
        }
    }
}

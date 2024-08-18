use std::iter;

use crate::{
    common::bit_subset64::BitSubset64,
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

use super::{bitset::BIG_INT, ref_cast::RefCast};

pub enum Sign {
    Positive = 1,
    Negative = -1,
}

pub struct JsBigIntHeader {
    len: isize,
}

pub type JsBigInt = FlexibleArray<u64, JsBigIntHeader>;

pub type JsBigIntRef<D> = Ref<JsBigInt, D>;

pub type JsBigIntMutRef<D> = MutRef<JsBigInt, D>;

impl FlexibleArrayHeader for JsBigIntHeader {
    fn len(&self) -> usize {
        self.len.unsigned_abs()
    }
}

impl<D: Dealloc> RefCast<D> for JsBigInt {
    const REF_SUBSET: BitSubset64<*const Block<JsBigInt, D>> = BIG_INT.cast();
}

pub fn new_big_int<M: Manager, I: ExactSizeIterator<Item = u64>>(
    m: M,
    sign: Sign,
    i: impl IntoIterator<IntoIter = I>,
) -> JsBigIntMutRef<M::Dealloc> {
    let items = i.into_iter();
    m.new(FlexibleArrayConstructor::new(
        JsBigIntHeader {
            len: (items.len() as isize) * sign as isize,
        },
        items,
    ))
}

pub fn zero<M: Manager>(m: M) -> JsBigIntMutRef<M::Dealloc> {
    new_big_int(m, Sign::Positive, iter::empty())
}

pub fn from_u64<M: Manager>(m: M, sign: Sign, n: u64) -> JsBigIntMutRef<M::Dealloc> {
    if n == 0 {
        return zero(m);
    }
    new_big_int(m, sign, iter::once(n))
}

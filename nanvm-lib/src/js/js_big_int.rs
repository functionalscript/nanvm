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

pub struct JsBigIntHeader {
    len: usize,
    ms: i64,
}

pub type JsBigInt = FlexibleArray<u64, JsBigIntHeader>;

pub type JsBigIntRef<D> = Ref<JsBigInt, D>;

pub type JsBigIntMutRef<D> = MutRef<JsBigInt, D>;

impl FlexibleArrayHeader for JsBigIntHeader {
    fn len(&self) -> usize {
        self.len
    }
}

impl<D: Dealloc> RefCast<D> for JsBigInt {
    const REF_SUBSET: BitSubset64<*const Block<JsBigInt, D>> = BIG_INT.cast();
}

pub fn new_big_int<M: Manager, I: ExactSizeIterator<Item = u64>>(
    m: M,
    ms: i64,
    i: impl IntoIterator<IntoIter = I>,
) -> JsBigIntMutRef<M::Dealloc> {
    let items = i.into_iter();
    m.new(FlexibleArrayConstructor::new(
        JsBigIntHeader {
            len: items.len(),
            ms,
        },
        items,
    ))
}

pub fn from_i64<M: Manager>(m: M, ms: i64) -> JsBigIntMutRef<M::Dealloc> {
    new_big_int(m, ms, iter::empty::<u64>())
}

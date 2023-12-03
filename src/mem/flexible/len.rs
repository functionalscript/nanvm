use core::marker::PhantomData;

use super::{header::FlexibleHeader, new::FlexibleNew};

#[repr(transparent)]
pub struct FlexibleLen<I> {
    len: usize,
    _0: PhantomData<I>,
}

impl<I> FlexibleHeader for FlexibleLen<I> {
    type Item = I;
    fn len(&self) -> usize {
        self.len
    }
}

impl<I: ExactSizeIterator> From<I> for FlexibleNew<FlexibleLen<I::Item>, I> {
    fn from(items: I) -> Self {
        FlexibleLen { len: items.len(), _0: PhantomData }.to_new(items)
    }
}

use core::marker::PhantomData;

use crate::mem::manager::Dealloc;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct AnyInternal<D: Dealloc>(pub u64, PhantomData<D>);

use crate::{
    common::cast::Cast,
    mem::{manager::Dealloc, ref_::Ref},
};

use super::{any::Any, any_internal::AnyInternal, ref_cast::RefCast, value_cast::ValueCast};

pub trait AnyCast<D: Dealloc>: Sized {
    unsafe fn has_same_type(any_internal: u64) -> bool;
    unsafe fn move_to_any_internal(self) -> u64;
    unsafe fn from_any_internal(any_internal: u64) -> Self;
    //
    #[inline(always)]
    fn move_to_any(self) -> Any<D> {
        unsafe { Any::from_internal(AnyInternal::new(self.move_to_any_internal())) }
    }
}

impl<D: Dealloc, T: ValueCast> AnyCast<D> for T
where
    u64: Cast<T>,
{
    #[inline(always)]
    unsafe fn has_same_type(any_internal: u64) -> bool {
        T::SUBSET.has(any_internal)
    }
    #[inline(always)]
    unsafe fn move_to_any_internal(self) -> u64 {
        T::SUBSET.from_value_typed(self)
    }
    #[inline(always)]
    unsafe fn from_any_internal(any_internal: u64) -> Self {
        T::SUBSET.get_value_typed(any_internal)
    }
}

impl<D: Dealloc, T: RefCast<D>> AnyCast<D> for Ref<T, D> {
    #[inline(always)]
    unsafe fn has_same_type(any_internal: u64) -> bool {
        T::REF_SUBSET.has(any_internal)
    }
    #[inline(always)]
    unsafe fn move_to_any_internal(self) -> u64 {
        T::REF_SUBSET.from_value_typed(self.move_to_internal())
    }
    #[inline(always)]
    unsafe fn from_any_internal(any_internal: u64) -> Self {
        Self::from_internal(T::REF_SUBSET.get_value_typed(any_internal))
    }
}

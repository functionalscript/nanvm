use crate::{
    common::cast::Cast,
    mem::{block::Block, manager::Dealloc, ref_::Ref},
};

use super::{
    any::Any, any_internal::AnyInternal, extension::Extension, extension_ref::ExtensionRef,
};

pub trait AnyCast<D: Dealloc>: Sized {
    unsafe fn is_type_of(u: u64) -> bool;
    unsafe fn move_to_any_internal(self) -> u64;
    unsafe fn from_any_internal(set: u64) -> Self;
    //
    #[inline(always)]
    fn move_to_any(self) -> Any<D> {
        unsafe { Any::from_internal(AnyInternal::new(self.move_to_any_internal())) }
    }
}

impl<D: Dealloc, T: Extension + Cast<u64>> AnyCast<D> for T
where
    u64: Cast<T>,
{
    #[inline(always)]
    unsafe fn is_type_of(u: u64) -> bool {
        T::SUBSET.has(u)
    }
    #[inline(always)]
    unsafe fn move_to_any_internal(self) -> u64 {
        T::SUBSET.from_value_typed(self)
    }
    #[inline(always)]
    unsafe fn from_any_internal(set: u64) -> Self {
        T::SUBSET.get_value_typed(set)
    }
}

impl<D: Dealloc, T: ExtensionRef<D>> AnyCast<D> for Ref<T, D> {
    #[inline(always)]
    unsafe fn is_type_of(u: u64) -> bool {
        T::REF_SUBSET.has(u)
    }
    #[inline(always)]
    unsafe fn move_to_any_internal(self) -> u64 {
        T::REF_SUBSET.from_value(self.move_to_internal() as u64)
    }
    #[inline(always)]
    unsafe fn from_any_internal(set: u64) -> Self {
        Self::from_internal(T::REF_SUBSET.get_value(set) as *mut _)
    }
}

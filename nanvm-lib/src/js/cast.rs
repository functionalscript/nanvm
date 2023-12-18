use super::{any::Any, any_internal::AnyInternal, extension::Extension};

pub trait Cast: Sized {
    unsafe fn is_type_of(u: u64) -> bool;
    unsafe fn move_to_any_internal(self) -> u64;
    unsafe fn from_any_internal(u: u64) -> Self;
    //
    #[inline(always)]
    fn move_to_any(self) -> Any {
        unsafe { Any::from_optional_base(AnyInternal::new(self.move_to_any_internal())) }
    }
}

impl<T: Extension> Cast for T {
    #[inline(always)]
    unsafe fn is_type_of(u: u64) -> bool {
        T::SUBSET.has(u)
    }
    #[inline(always)]
    unsafe fn move_to_any_internal(self) -> u64 {
        T::move_to_superposition(self) | T::SUBSET.tag
    }
    #[inline(always)]
    unsafe fn from_any_internal(u: u64) -> Self {
        T::from_superposition(u & T::SUBSET.superposition())
    }
}

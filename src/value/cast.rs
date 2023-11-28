use super::{internal::Internal, tag::Tag, unknown::Unknown};

pub trait Cast: Sized {
    unsafe fn cast_is(u: u64) -> bool;
    unsafe fn move_to_unknown_internal(self) -> u64;
    unsafe fn from_unknown_internal(u: u64) -> Self;
    //
    #[inline(always)]
    fn unknown(self) -> Unknown {
        unsafe { Unknown::from_internal(Internal(self.move_to_unknown_internal())) }
    }
}

impl<T: Tag> Cast for T {
    #[inline(always)]
    unsafe fn cast_is(u: u64) -> bool {
        T::SUBSET.has(u)
    }
    #[inline(always)]
    unsafe fn move_to_unknown_internal(self) -> u64 {
        T::move_to_superposition(self) | T::SUBSET.tag
    }
    #[inline(always)]
    unsafe fn from_unknown_internal(u: u64) -> Self {
        T::from_superposition(u & T::SUBSET.superposition())
    }
}

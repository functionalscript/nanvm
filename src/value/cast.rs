use super::{tag::Tag, unknown::Unknown};

pub trait Cast: Sized {
    unsafe fn cast_is(u: u64) -> bool;
    unsafe fn cast_into(self) -> u64;
    unsafe fn cast_from(u: u64) -> Self;
    //
    #[inline(always)]
    fn unknown(self) -> Unknown {
        unsafe { Unknown::from_u64(self.cast_into()) }
    }
}

impl<T: Tag> Cast for T {
    #[inline(always)]
    unsafe fn cast_is(u: u64) -> bool {
        T::SUBSET.has(u)
    }
    #[inline(always)]
    unsafe fn cast_into(self) -> u64 {
        T::move_to_unknown_raw(self) | T::SUBSET.tag
    }
    #[inline(always)]
    unsafe fn cast_from(u: u64) -> Self {
        T::from_unknown_raw(u & T::SUBSET.superposition())
    }
}

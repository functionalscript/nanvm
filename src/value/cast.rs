use super::{tag::Tag, unknown::Unknown};

pub trait Cast: Sized {
    fn cast_is(u: u64) -> bool;
    fn cast_into(self) -> u64;
    fn cast_from(u: u64) -> Self;
    //
    #[inline(always)]
    fn unknown(self) -> Unknown {
        Unknown::from_u64(self.cast_into())
    }
}

impl<T: Tag> Cast for T {
    #[inline(always)]
    fn cast_is(u: u64) -> bool {
        T::SUBSET.has(u)
    }
    #[inline(always)]
    fn cast_into(self) -> u64 {
        T::to_unknown_raw(self) | T::SUBSET.tag
    }
    #[inline(always)]
    fn cast_from(u: u64) -> Self {
        T::from_unknown_raw(u & T::SUBSET.superposition())
    }
}

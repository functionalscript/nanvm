use crate::{
    common::bit_subset64::BitSubset64,
    container::{Base, Container, Info, Rc},
};

use super::{bitset::RC_SUBSET_SUPERPOSITION, extension::Extension};

pub trait ExtensionRc: Info {
    const RC_SUBSET: BitSubset64;
    const _0: () = assert!(Self::RC_SUBSET.superposition() == RC_SUBSET_SUPERPOSITION);
    unsafe fn delete(p: *mut Base) {
        Container::delete(p as *mut Container<Self>);
    }
}

impl<T: ExtensionRc> Extension for Rc<T> {
    const SUBSET: BitSubset64 = T::RC_SUBSET;
    #[inline(always)]
    unsafe fn move_to_superposition(self) -> u64 {
        self.move_to_optional_base() as u64
    }
    #[inline(always)]
    unsafe fn from_superposition(u: u64) -> Self {
        Self::from_optional_base(u as *mut Container<T>)
    }
}

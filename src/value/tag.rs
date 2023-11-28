use crate::{
    common::bit_subset64::BitSubset64,
    container::{Base, Container, Info, Rc},
};

use super::extension::PTR_SUBSET_SUPERPOSITION;

pub trait Tag {
    const SUBSET: BitSubset64;
    unsafe fn move_to_superposition(self) -> u64;
    unsafe fn from_superposition(u: u64) -> Self;
}

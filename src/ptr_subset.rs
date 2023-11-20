use std::marker::PhantomData;

use crate::{container::{Clean, Container}, bit_subset64::BitSubset64};

pub struct PtrSubset<T: Clean>(pub BitSubset64, pub PhantomData<T>);

impl<T: Clean> PtrSubset<T> {
    pub fn update<const ADD: bool>(&self, v: u64) {
        let v = v & self.0.superposition();
        if v == 0 {
            return;
        }
        unsafe {
            Container::update::<ADD>(v as *mut Container<T>);
        }
    }
}
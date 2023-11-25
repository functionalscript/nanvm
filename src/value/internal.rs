use crate::{
    container::{Base, OptionalBase},
    ptr_subset::PTR_SUBSET_SUPERPOSITION,
};

use super::extension::{OBJECT, PTR, STRING};

#[repr(transparent)]
#[derive(Clone)]
pub struct Internal(u64);

impl OptionalBase for Internal {
    unsafe fn get_base(&self) -> Option<*mut Base> {
        let v = self.0;
        if !PTR.has(v) {
            return None;
        }
        let i = v & PTR_SUBSET_SUPERPOSITION;
        Some(i as *mut Base)
    }
    unsafe fn dealloc(&self, base: *mut Base) {
        if STRING.subset().has(self.0) {
            STRING.dealloc(base);
        } else {
            OBJECT.dealloc(base);
        }
    }
}

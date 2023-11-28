use crate::container::{Base, OptionalBase};

use super::{
    bitset::{RC, RC_SUBSET_SUPERPOSITION, STRING},
    extension_rc::TagRc,
    object::ObjectHeader,
    string::StringHeader,
};

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Internal(pub u64);

impl OptionalBase for Internal {
    unsafe fn get_base(&self) -> Option<*mut Base> {
        let v = self.0;
        if !RC.has(v) {
            return None;
        }
        let i = v & RC_SUBSET_SUPERPOSITION;
        Some(i as *mut Base)
    }
    unsafe fn delete(&self, base: *mut Base) {
        if STRING.has(self.0) {
            StringHeader::delete(base);
        } else {
            ObjectHeader::delete(base);
        }
    }
}

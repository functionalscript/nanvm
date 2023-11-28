use crate::container::{Base, OptionalBase};

use super::{
    extension::{PTR_SUBSET_SUPERPOSITION, RC, STRING},
    object::ObjectHeader,
    string::StringHeader,
    tag_rc::TagRc,
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
        let i = v & PTR_SUBSET_SUPERPOSITION;
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

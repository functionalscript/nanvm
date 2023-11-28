use crate::container::{Base, OptionalBase};

use super::{
    bitset::{RC, RC_SUBSET_SUPERPOSITION, STRING},
    extension_rc::ExtensionRc,
    object::ObjectHeader,
    string::StringHeader,
};

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct AnyInternal(pub u64);

impl OptionalBase for AnyInternal {
    unsafe fn get_base(&self) -> Option<*mut Base> {
        let v = self.0;
        if !RC.has(v) {
            return None;
        }
        Some((v & RC_SUBSET_SUPERPOSITION) as *mut Base)
    }
    unsafe fn delete(&self, base: *mut Base) {
        if STRING.has(self.0) {
            StringHeader::delete(base);
        } else {
            ObjectHeader::delete(base);
        }
    }
}

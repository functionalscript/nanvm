use super::{object::Object, ref_::update::RefUpdate};

pub trait BlockHeader {
    unsafe fn ref_update(&self, i: RefUpdate) -> isize;
    unsafe fn get_object<T: Object>(&mut self) -> &mut T;
    unsafe fn delete<T: Object>(&mut self);
}

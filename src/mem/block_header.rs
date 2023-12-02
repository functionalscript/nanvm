use super::{object::Object, rc_update::RcUpdate};

pub trait BlockHeader {
    unsafe fn rc_update(&self, i: RcUpdate) -> isize;
    unsafe fn get_object<T: Object>(&mut self) -> &mut T;
    unsafe fn delete<T: Object>(&mut self);
}

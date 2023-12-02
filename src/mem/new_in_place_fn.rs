use super::object::Object;

pub trait NewInPlaceFn {
    type Result: Object;
    fn result_size(&self) -> usize;
    unsafe fn new_in_place(self, p: *mut Self::Result);
}

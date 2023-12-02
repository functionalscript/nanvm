use super::object::Object;

pub trait NewInPlace {
    type Object: Object;
    fn size(&self) -> usize;
    unsafe fn new_in_place(self, p: *mut Self::Object);
}

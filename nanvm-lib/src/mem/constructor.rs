use super::object::Object;

pub trait Constructor {
    type Object: Object;
    fn new_size(&self) -> usize;
    unsafe fn construct(self, p: *mut Self::Object);
}

pub trait Assign {
    type Object: Object;
    fn new_size(&self) -> usize;
    unsafe fn assign(self, p: *mut Self::Object);
}

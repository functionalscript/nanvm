use super::object::Object;

pub trait Constructor {
    type Result: Object;
    fn result_size(&self) -> usize;
    unsafe fn construct(self, p: *mut Self::Result);
}

use crate::container::Containable;

#[repr(C)]
pub struct String16();

impl Containable for String16 {
    fn clean(&mut self) {}
}

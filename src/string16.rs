use crate::containable::Containable;

pub struct String16();

impl Containable for String16 {
    type Item = u16;
}

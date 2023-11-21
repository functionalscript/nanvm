use crate::container::Containable;

pub struct Object {}

impl Containable for Object {
    fn clean(&mut self) {}
}

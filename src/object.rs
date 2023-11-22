use crate::{containable::Containable, container_ref::Ref, string16::String16, value::Value};

pub struct Object();

impl Containable for Object {
    type Item = (Ref<String16>, Value);
}

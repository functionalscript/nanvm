use crate::{
    container::{Containable, Ref},
    string16::String16,
    value::Value,
};

pub struct Object();

impl Containable for Object {
    type Item = (Ref<String16>, Value);
}

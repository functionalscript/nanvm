use crate::{common::default::default, js::any::Any, mem::manager::Dealloc};

pub fn to_json(array: &Any<impl Dealloc>) -> String {
    default()
}

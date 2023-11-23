use crate::{
    container::{Header, Ref},
    string::StringHeader,
    value::Value,
};

pub struct ObjectHeader();

impl Header for ObjectHeader {
    type Item = (Ref<StringHeader>, Value);
}

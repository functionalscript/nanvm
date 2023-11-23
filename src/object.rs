use crate::{
    container::{Info, Ref},
    string::StringHeader,
    value::Value,
};

pub struct ObjectHeader(usize);

impl Info for ObjectHeader {
    type Item = (Ref<StringHeader>, Value);
}

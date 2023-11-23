use crate::{
    container::{Header, Ref},
    string::StringHeader,
    value::Value,
};

pub struct ObjectHeader(usize);

impl Header for ObjectHeader {
    type Item = (Ref<StringHeader>, Value);
    fn len(&self) -> usize {
        self.0
    }
}

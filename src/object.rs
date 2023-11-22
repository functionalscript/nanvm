use crate::{
    container::{Info, Ref},
    string::StringInfo,
    value::Value,
};

pub struct ObjectInfo();

impl Info for ObjectInfo {
    type Item = (Ref<StringInfo>, Value);
}

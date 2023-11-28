use crate::{
    common::bit_subset64::BitSubset64,
    container::{Container, Info, Rc},
    value::unknown::Unknown,
};

use super::{extension::OBJECT, tag::TagRc};

pub struct ObjectHeader();

impl Info for ObjectHeader {
    type Item = (Unknown, Unknown);
}

pub type ObjectContainer = Container<ObjectHeader>;

pub type ObjectRc = Rc<ObjectHeader>;

impl TagRc for ObjectHeader {
    const RC_SUBSET: BitSubset64 = OBJECT;
}

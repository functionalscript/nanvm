use crate::container::Info;

pub struct StringHeader();

impl Info for StringHeader {
    type Item = u16;
}

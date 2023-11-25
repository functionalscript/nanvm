use super::Base;

pub trait OptionalBase: Clone {
    fn get_base(&self) -> Option<&mut Base>;
    fn dealloc(base: &mut Base);
}

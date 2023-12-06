use super::Object;

pub trait ObjectHolder {
    type Object: Object;
    fn object(&self) -> &Self::Object;
}

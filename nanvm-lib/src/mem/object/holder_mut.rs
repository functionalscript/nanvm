use super::holder::ObjectHolder;

pub trait ObjectHolderMut: ObjectHolder {
    fn mut_object(&mut self) -> &mut Self::Object;
}

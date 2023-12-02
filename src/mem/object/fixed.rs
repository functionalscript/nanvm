use super::Object;

#[repr(transparent)]
pub struct Fixed<T>(pub T);

impl<T> Object for Fixed<T> {}

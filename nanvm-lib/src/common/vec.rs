use super::default::default;

pub fn new_resize<T: Default + Clone>(size: usize) -> Vec<T> {
    let mut vec = Vec::with_capacity(size);
    vec.resize(size, default());
    vec
}

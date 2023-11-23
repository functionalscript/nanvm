pub trait Header {
    type Item;
    fn len(&self) -> usize;
}

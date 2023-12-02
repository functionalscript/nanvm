pub trait FlexibleArrayHeader: Sized {
    type Item;
    fn len(&self) -> usize;
}

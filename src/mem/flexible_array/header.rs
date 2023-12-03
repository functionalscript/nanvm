use super::new::FlexibleArrayNew;

pub trait FlexibleArrayHeader: Sized {
    // required
    type Item;
    fn len(&self) -> usize;
    //
    fn to_new<I: Iterator<Item = Self::Item>>(self, items: I) -> FlexibleArrayNew<Self, I> {
        FlexibleArrayNew::new(self, items)
    }
}

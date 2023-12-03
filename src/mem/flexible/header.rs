use super::new::FlexibleNew;

pub trait FlexibleHeader: Sized {
    // required
    type Item;
    fn len(&self) -> usize;
    //
    fn to_new<I: Iterator<Item = Self::Item>>(self, i: I) -> FlexibleNew<Self, I> {
        FlexibleNew::new(self, i)
    }
}

use super::{
    super::{super::common::ref_mut::RefMut, new_in_place_fn::NewInPlaceFn},
    header::FlexibleArrayHeader,
    FlexibleArray,
};

struct FlexibleArrayNew<H: FlexibleArrayHeader, I: Iterator<Item = H::Item>> {
    header: H,
    items: I,
}

impl<H: FlexibleArrayHeader, I: Iterator<Item = H::Item>> NewInPlaceFn for FlexibleArrayNew<H, I> {
    type Result = FlexibleArray<H>;
    fn result_size(&self) -> usize {
        Self::Result::flexible_array_size(self.header.len())
    }
    unsafe fn new_in_place(self, p: *mut Self::Result) {
        let v = &mut *p;
        v.header.as_mut_ptr().write(self.header);
        let mut src = self.items;
        for dst in v.get_items_mut() {
            dst.as_mut_ptr().write(src.next().unwrap());
        }
    }
}

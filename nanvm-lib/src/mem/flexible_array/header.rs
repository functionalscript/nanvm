use super::new::FlexibleArrayNew;

pub trait FlexibleArrayHeader: Sized {
    // required
    type Item;
    fn len(&self) -> usize;
    //
    #[inline(always)]
    fn to_new<I: Iterator<Item = Self::Item>>(self, items: I) -> FlexibleArrayNew<Self, I> {
        FlexibleArrayNew::new(self, items)
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{flexible_array::FlexibleArray, new_in_place::NewInPlace};

    use super::FlexibleArrayHeader;

    struct H();

    impl FlexibleArrayHeader for H {
        type Item = u8;
        fn len(&self) -> usize {
            5
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        let x = H();
        let y = x.to_new([1, 2, 3, 4, 5].iter().copied());
        let mut buffer = [0u8; 5];
        unsafe { y.new_in_place(buffer.as_mut_ptr() as *mut FlexibleArray<_>) };
        assert_eq!(buffer, [1, 2, 3, 4, 5]);
    }
}

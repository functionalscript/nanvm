use super::constructor::FlexibleArrayConstructor;

pub trait FlexibleArrayHeader: Sized {
    // required
    type Item;
    fn len(&self) -> usize;
    //
    #[inline(always)]
    fn constructor<I: Iterator<Item = Self::Item>>(
        self,
        items: I,
    ) -> FlexibleArrayConstructor<Self, I> {
        FlexibleArrayConstructor::new(self, items)
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{constructor::Constructor, flexible_array::FlexibleArray};

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
        let y = x.constructor([1, 2, 3, 4, 5].iter().copied());
        let mut buffer = [0u8; 5];
        unsafe { y.construct(buffer.as_mut_ptr() as *mut FlexibleArray<_>) };
        assert_eq!(buffer, [1, 2, 3, 4, 5]);
    }
}

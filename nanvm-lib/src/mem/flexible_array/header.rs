use super::constructor::FlexibleArrayConstructor;

pub trait FlexibleArrayHeader: Sized {
    // required
    fn len(&self) -> usize;
    //
    #[inline(always)]
    fn constructor<I: Iterator>(
        self,
        items: I,
    ) -> FlexibleArrayConstructor<Self, I> {
        FlexibleArrayConstructor::new(self, items)
    }
}

impl FlexibleArrayHeader for usize {
    #[inline(always)]
    fn len(&self) -> usize {
        *self
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{constructor::Constructor, object::Object, flexible_array::{FlexibleArray, constructor::FlexibleArrayConstructor}};

    use super::FlexibleArrayHeader;

    struct H();

    impl FlexibleArrayHeader for H {
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
        unsafe { y.construct(buffer.as_mut_ptr() as *mut FlexibleArray<u8, H>) };
        assert_eq!(buffer, [1, 2, 3, 4, 5]);
    }

    #[repr(C)]
    struct Item(*mut u8);

    impl Drop for Item {
        fn drop(&mut self) {
            unsafe {
                *self.0 += 1;
            }
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_len() {
        let mut buffer = [0usize; 5];

        let mut i: u8 = 0;
        {
            let p = &mut i as *mut _;
            let v = [Item(p), Item(p), Item(p), Item(p)];
            let x: FlexibleArrayConstructor<_, _> = v.into_iter().into();
            unsafe { x.construct(buffer.as_ptr() as *mut _) }
            assert_eq!(i, 0);
            assert_eq!(buffer[0], 4);
            assert_eq!(p, buffer[1] as *mut _);
            assert_eq!(p, buffer[2] as *mut _);
            assert_eq!(p, buffer[3] as *mut _);
            assert_eq!(p, buffer[4] as *mut _);
            let px = buffer.as_mut_ptr() as *mut FlexibleArray<Item, usize>;
            unsafe { (*px).object_drop() };
            assert_eq!(i, 4);
        }
        assert_eq!(i, 4);
    }
}

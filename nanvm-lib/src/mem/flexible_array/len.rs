use core::marker::PhantomData;

use super::{constructor::FlexibleArrayConstructor, header::FlexibleArrayHeader};

#[repr(transparent)]
pub struct FlexibleArrayLen {
    len: usize,
}

impl FlexibleArrayHeader for FlexibleArrayLen {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }
}

impl<I: ExactSizeIterator> From<I> for FlexibleArrayConstructor<FlexibleArrayLen, I> {
    #[inline(always)]
    fn from(items: I) -> Self {
        FlexibleArrayLen {
            len: items.len(),
        }
        .constructor(items)
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{
        constructor::Constructor,
        flexible_array::{
            constructor::FlexibleArrayConstructor, len::FlexibleArrayLen, FlexibleArray,
        },
        object::Object,
    };

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
    fn test() {
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
            let px = buffer.as_mut_ptr() as *mut FlexibleArray<FlexibleArrayLen, Item>;
            unsafe { (*px).object_drop() };
            assert_eq!(i, 4);
        }
        assert_eq!(i, 4);
    }
}

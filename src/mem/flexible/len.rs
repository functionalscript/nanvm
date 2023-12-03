use core::marker::PhantomData;

use super::{header::FlexibleHeader, new::FlexibleNew};

#[repr(transparent)]
pub struct FlexibleLen<I> {
    len: usize,
    _0: PhantomData<I>,
}

impl<I> FlexibleHeader for FlexibleLen<I> {
    type Item = I;
    fn len(&self) -> usize {
        self.len
    }
}

impl<I: ExactSizeIterator> From<I> for FlexibleNew<FlexibleLen<I::Item>, I> {
    fn from(items: I) -> Self {
        FlexibleLen {
            len: items.len(),
            _0: PhantomData,
        }
        .to_new(items)
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{
        flexible::{len::FlexibleLen, new::FlexibleNew, Flexible},
        new_in_place::NewInPlace,
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
            let x: FlexibleNew<_, _> = v.into_iter().into();
            unsafe { x.new_in_place(buffer.as_ptr() as *mut _) }
            assert_eq!(i, 0);
            assert_eq!(buffer[0], 4);
            assert_eq!(p, buffer[1] as *mut _);
            assert_eq!(p, buffer[2] as *mut _);
            assert_eq!(p, buffer[3] as *mut _);
            assert_eq!(p, buffer[4] as *mut _);
            let px = buffer.as_mut_ptr() as *mut Flexible<FlexibleLen<Item>>;
            unsafe { (*px).object_drop_in_place() };
            assert_eq!(i, 4);
        }
        assert_eq!(i, 4);
    }
}

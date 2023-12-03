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

    use crate::mem::{flexible::new::FlexibleNew, new_in_place::NewInPlace};

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
        let buffer = [0usize; 5];

        let mut i: u8 = 0;
        let p = &mut i as *mut _;
        let v = [Item(p), Item(p), Item(p), Item(p)];
        let x: FlexibleNew<_, _> = v.into_iter().into();
        unsafe { x.new_in_place(buffer.as_ptr() as *mut _) }
        assert_eq!(buffer[0], 4);
    }
}

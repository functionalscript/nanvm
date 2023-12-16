use core::{
    mem::{align_of, size_of},
    ptr::drop_in_place,
};

/// Object properties
pub trait Object: Sized {
    const OBJECT_ALIGN: usize = align_of::<Self>();
    #[inline(always)]
    fn object_size(&self) -> usize {
        size_of::<Self>()
    }
    #[inline(always)]
    unsafe fn object_drop(&mut self) {
        drop_in_place(self)
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::Object;

    #[repr(transparent)]
    struct A(i32);

    impl Object for A {}

    impl Drop for A {
        fn drop(&mut self) {
            self.0 += 1;
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        assert_eq!(A(5).object_size(), 4);
        let mut a = A(5);
        unsafe { a.object_drop() };
        assert_eq!(a.0, 6);
    }
}

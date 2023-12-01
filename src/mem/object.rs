use core::{
    mem::{align_of, size_of},
    ptr::drop_in_place,
};

/// Object properties
pub trait Object {
    const ALIGN: usize;
    fn size(&self) -> usize;
    unsafe fn drop_in_place(&mut self);
}

impl<T> Object for T {
    const ALIGN: usize = align_of::<T>();
    fn size(&self) -> usize {
        size_of::<T>()
    }
    unsafe fn drop_in_place(&mut self) {
        drop_in_place(self)
    }
}

#[cfg(test)]
mod test {
    use core::{
        mem::forget,
        sync::atomic::{AtomicIsize, Ordering},
    };

    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[test]
    #[wasm_bindgen_test]
    fn test_object() {
        let a = 5;
        assert_eq!(a.size(), 4);
    }

    struct X<'a>(&'a AtomicIsize);

    impl Drop for X<'_> {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_object_drop_in_place() {
        let a = AtomicIsize::new(5);
        {
            let mut x = X(&a);
            unsafe { x.drop_in_place() };
            assert_eq!(a.load(Ordering::Relaxed), 6);
            forget(x);
        }
        assert_eq!(a.load(Ordering::Relaxed), 6);
    }
}

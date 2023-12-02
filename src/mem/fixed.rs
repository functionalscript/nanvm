use super::Object;

#[repr(transparent)]
pub struct Fixed<T>(pub T);

impl<T> Object for Fixed<T> {}

#[cfg(test)]
mod test {
    use core::{
        mem::forget,
        sync::atomic::{AtomicIsize, Ordering},
    };

    use wasm_bindgen_test::wasm_bindgen_test;

    use super::{Fixed, *};

    #[test]
    #[wasm_bindgen_test]
    fn test_object() {
        let a = Fixed(5);
        assert_eq!(a.object_size(), 4);
    }

    struct X<'a>(&'a AtomicIsize);

    impl Object for X<'_> {}

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
            unsafe { x.object_drop_in_place() };
            assert_eq!(a.load(Ordering::Relaxed), 6);
            forget(x);
        }
        assert_eq!(a.load(Ordering::Relaxed), 6);
    }
}

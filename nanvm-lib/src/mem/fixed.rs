use super::{constructor::Constructor, object::Object};

#[repr(transparent)]
#[derive(Debug)]
pub struct Fixed<T>(pub T);

impl<T> Object for Fixed<T> {}

impl<T> Constructor for Fixed<T> {
    type Result = Fixed<T>;
    #[inline(always)]
    fn result_size(&self) -> usize {
        Self::Result::object_size(self)
    }
    #[inline(always)]
    unsafe fn construct(self, p: *mut Self::Result) {
        p.write(self);
    }
}

#[cfg(test)]
mod test {
    use core::{
        mem::forget,
        ptr::null,
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
            unsafe { x.object_drop() };
            assert_eq!(a.load(Ordering::Relaxed), 6);
            forget(x);
        }
        assert_eq!(a.load(Ordering::Relaxed), 6);
    }

    struct Y(*const AtomicIsize);

    impl Drop for Y {
        fn drop(&mut self) {
            unsafe { (*self.0).fetch_add(1, Ordering::Relaxed) };
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_fixed_object_drop_in_place() {
        let a = AtomicIsize::new(5);
        {
            let mut x = Fixed(Y(&a));
            unsafe { x.object_drop() };
            assert_eq!(a.load(Ordering::Relaxed), 6);
            forget(x);
        }
        assert_eq!(a.load(Ordering::Relaxed), 6);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_new_in_place() {
        let a = AtomicIsize::new(5);
        {
            let x = Fixed(Y(&a as *const AtomicIsize));
            {
                let mut y = Fixed(Y(null()));
                unsafe { x.construct(&mut y) };
                assert_eq!(a.load(Ordering::Relaxed), 5);
            }
            assert_eq!(a.load(Ordering::Relaxed), 6);
        }
        assert_eq!(a.load(Ordering::Relaxed), 6);
    }
}

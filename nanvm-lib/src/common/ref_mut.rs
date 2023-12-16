pub trait RefMut {
    // a better name is `as_mut_ptr` but it's already used by `Slice`.
    unsafe fn to_mut_ptr(&mut self) -> *mut Self {
        self as *mut Self
    }
    fn modify(&mut self, f: impl FnOnce(Self) -> Self)
    where
        Self: Sized,
    {
        unsafe {
            let p = self.to_mut_ptr();
            p.write(f(p.read()));
        };
    }
}

impl<T> RefMut for T {}

#[cfg(test)]
mod test {
    use core::sync::atomic::{AtomicIsize, Ordering};

    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    struct A<'a>(i32, &'a AtomicIsize);

    impl Drop for A<'_> {
        fn drop(&mut self) {
            self.1.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_move_update() {
        let x = AtomicIsize::new(0);
        {
            let mut a = A(5, &x);
            let mut i = 0;
            a.modify(|mut a| {
                a.0 += 1;
                i += 1;
                a
            });
            assert_eq!(a.0, 6);
            assert_eq!(i, 1);
            assert_eq!(x.load(Ordering::Relaxed), 0);
            a.modify(|a| A(a.0 - 2, a.1));
            assert_eq!(a.0, 4);
            assert_eq!(x.load(Ordering::Relaxed), 1);
        }
        assert_eq!(x.load(Ordering::Relaxed), 2);
    }
}

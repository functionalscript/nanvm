use core::ptr::{read, write};

trait AsMutPtr<T> {
    fn as_mut_ptr(&mut self) -> *mut T;
}

impl<T> AsMutPtr<T> for T {
    fn as_mut_ptr(&mut self) -> *mut T {
        self as *mut T
    }
}

pub fn move_update<T>(src: &mut T, f: impl FnOnce(T) -> T) {
    unsafe { write(src.as_mut_ptr(), f(read(src))) };
}

#[cfg(test)]
mod test {
    use std::sync::atomic::{AtomicIsize, Ordering};

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
            move_update(&mut a, |mut a| {
                a.0 += 1;
                i += 1;
                a
            });
            assert_eq!(a.0, 6);
            assert_eq!(i, 1);
            assert_eq!(x.load(Ordering::Relaxed), 0);
            move_update(&mut a, |a| A(a.0 - 2, a.1));
            assert_eq!(a.0, 4);
            assert_eq!(x.load(Ordering::Relaxed), 1);
        }
        assert_eq!(x.load(Ordering::Relaxed), 2);
    }
}

use core::{
    alloc::Layout,
    mem::{align_of, size_of},
    ptr::drop_in_place,
    slice::from_raw_parts_mut,
};

use super::typed_layout::TypedLayout;

/// Object properties
pub trait Object: Sized {
    const ALIGN: usize = align_of::<Self>();
    fn size(&self) -> usize {
        size_of::<Self>()
    }
    unsafe fn drop_in_place(&mut self) {
        drop_in_place(self)
    }
}

#[repr(transparent)]
pub struct Fixed<T>(pub T);

impl<T> Object for Fixed<T> {}

pub trait FasHeader: Sized {
    type Item;
    fn len(&self) -> usize;
    const LAYOUT: TypedLayout<Self, Self::Item> = TypedLayout::align_to(align_of::<Self::Item>());
    fn get_items_mut(&mut self) -> &mut [Self::Item] {
        unsafe { from_raw_parts_mut(Self::LAYOUT.to_end(self), self.len()) }
    }
}

#[repr(transparent)]
pub struct Fas<T: FasHeader>(pub T);

impl<T: FasHeader> Object for Fas<T> {
    const ALIGN: usize = T::LAYOUT.align;
    fn size(&self) -> usize {
        T::LAYOUT.size + self.0.len() * size_of::<T::Item>()
    }
    unsafe fn drop_in_place(&mut self) {
        drop_in_place(self.0.get_items_mut());
        drop_in_place(self);
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
        let a = Fixed(5);
        assert_eq!(a.size(), 4);
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
            unsafe { x.drop_in_place() };
            assert_eq!(a.load(Ordering::Relaxed), 6);
            forget(x);
        }
        assert_eq!(a.load(Ordering::Relaxed), 6);
    }
}

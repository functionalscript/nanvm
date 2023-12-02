use core::{
    mem::{align_of, size_of},
    ptr::drop_in_place,
    slice::from_raw_parts_mut,
};

use crate::mem::field_layout::FieldLayout;

use super::Object;

pub trait VariableHeader: Sized {
    // required
    type Item;
    fn len(&self) -> usize;
}

#[repr(transparent)]
pub struct Variable<T: VariableHeader>(pub T);

impl<T: VariableHeader> Variable<T> {
    const VARIABLE_HEADER_LAYOUT: FieldLayout<T, T::Item> =
        FieldLayout::align_to(align_of::<T::Item>());
    pub fn get_items_mut(&mut self) -> &mut [T::Item] {
        unsafe {
            from_raw_parts_mut(
                Self::VARIABLE_HEADER_LAYOUT.to_adjacent(&mut self.0),
                self.0.len(),
            )
        }
    }
}

impl<T: VariableHeader> Object for Variable<T> {
    const OBJECT_ALIGN: usize = Self::VARIABLE_HEADER_LAYOUT.align;
    fn object_size(&self) -> usize {
        Self::VARIABLE_HEADER_LAYOUT.size + self.0.len() * size_of::<T::Item>()
    }
    unsafe fn object_drop_in_place(&mut self) {
        drop_in_place(self.get_items_mut());
        drop_in_place(self);
    }
}

#[cfg(test)]
mod test {
    use core::{
        fmt::Debug,
        marker::PhantomData,
        mem::{forget, size_of},
        sync::atomic::{AtomicUsize, Ordering},
    };

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::object::Object;

    use super::{Variable, VariableHeader};

    struct X<H, I>(H, PhantomData<I>);

    impl<H: Into<usize> + Copy, I> VariableHeader for X<H, I> {
        type Item = I;
        fn len(&self) -> usize {
            self.0.into()
        }
    }

    #[repr(C)]
    struct Y<H, I, const N: usize> {
        len: H,
        items: [I; N],
    }

    fn ptr<T>(x: &mut T) -> *mut T {
        x as *mut T
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_u8_5() {
        let mut y = Y::<u16, u8, 5> {
            len: 5,
            items: [42, 43, 44, 45, 46],
        };
        let v = ptr(&mut y) as *mut Variable<X<u16, u8>>;
        unsafe {
            assert_eq!((*v).0.len(), 5);
            assert_eq!((*v).object_size(), 7);
            let items = (*v).get_items_mut();
            assert_eq!(items, &[42, 43, 44, 45, 46]);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_u32_3() {
        let mut y = Y::<u16, u32, 3> {
            len: 3,
            items: [42, 43, 44],
        };
        let v = ptr(&mut y) as *mut Variable<X<u16, u32>>;
        unsafe {
            assert_eq!((*v).0.len(), 3);
            assert_eq!((*v).object_size(), 16);
            let items = (*v).get_items_mut();
            assert_eq!(items, &[42, 43, 44]);
        }
    }

    fn generic_test<
        H: Into<usize> + Copy + TryFrom<usize>,
        I: PartialEq + Debug + Copy,
        const N: usize,
    >(
        items: [I; N],
        size: usize,
    ) {
        let old = items;
        let mut y = Y::<H, I, N> {
            len: unsafe { N.try_into().unwrap_unchecked() },
            items,
        };
        let v = ptr(&mut y) as *mut Variable<X<H, I>>;
        unsafe {
            assert_eq!((*v).0.len(), N);
            assert_eq!((*v).object_size(), size);
            assert_eq!(&*(*v).get_items_mut(), &old[..]);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test1() {
        generic_test::<u8, u8, 1>([42], 2);
        generic_test::<u16, u32, 6>([42, 56, 78, 90, 101, 102], 28);
        generic_test::<u16, u8, 3>([90, 101, 102], 5);
    }

    #[repr(transparent)]
    struct DropCount(*const AtomicUsize);

    impl Drop for DropCount {
        fn drop(&mut self) {
            unsafe {
                (*self.0).fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    impl VariableHeader for DropCount {
        type Item = DropCount;
        fn len(&self) -> usize {
            3
        }
    }

    #[repr(C)]
    struct DropCountX {
        header: DropCount,
        items: [DropCount; 3],
    }

    #[test]
    #[wasm_bindgen_test]
    fn drop_test() {
        let i = AtomicUsize::new(0);
        {
            let mut x = DropCountX {
                header: DropCount(&i as *const AtomicUsize),
                items: [
                    DropCount(&i as *const AtomicUsize),
                    DropCount(&i as *const AtomicUsize),
                    DropCount(&i as *const AtomicUsize),
                ],
            };
            let v = ptr(&mut x) as *mut Variable<DropCount>;
            unsafe {
                assert_eq!((*v).0.len(), 3);
                assert_eq!((*v).object_size(), size_of::<DropCountX>());
                assert_eq!((*v).object_size(), size_of::<DropCount>() * 4);
                let a = &*(*v).get_items_mut();
                assert_eq!(a.len(), 3);
                assert_eq!(a[0].0, &i as *const AtomicUsize);
                assert_eq!(a[1].0, &i as *const AtomicUsize);
                assert_eq!(a[2].0, &i as *const AtomicUsize);
            }
            assert_eq!(i.load(Ordering::Relaxed), 0);
            unsafe { (*v).object_drop_in_place() };
            assert_eq!(i.load(Ordering::Relaxed), 4);
            //forget(x);
        }
        // dro() called twice if forget to forget :-)
        assert_eq!(i.load(Ordering::Relaxed), 8);
    }
}

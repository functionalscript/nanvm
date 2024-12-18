pub mod constructor;
pub mod header;
use core::{
    marker::PhantomData,
    mem::{align_of, size_of},
    ptr::drop_in_place,
    slice::{from_raw_parts, from_raw_parts_mut},
};

use self::header::FlexibleArrayHeader;

use super::{field_layout::FieldLayout, object::Object};

#[repr(transparent)]
#[derive(Debug)]
pub struct FlexibleArray<I, T: FlexibleArrayHeader = usize> {
    pub header: T,
    _0: PhantomData<I>,
}

impl<I, T: FlexibleArrayHeader> FlexibleArray<I, T> {
    const FLEXIBLE_HEADER_LAYOUT: FieldLayout<T, I> = FieldLayout::align_to(align_of::<I>());
    #[inline(always)]
    pub fn items(&self) -> &[I] {
        unsafe {
            from_raw_parts(
                Self::FLEXIBLE_HEADER_LAYOUT.to_adjacent(&self.header),
                self.header.len(),
            )
        }
    }
    #[inline(always)]
    pub fn items_mut(&mut self) -> &mut [I] {
        unsafe {
            from_raw_parts_mut(
                Self::FLEXIBLE_HEADER_LAYOUT.into_adjacent_mut(&mut self.header),
                self.header.len(),
            )
        }
    }
    #[inline(always)]
    pub const fn flexible_size(len: usize) -> usize {
        Self::FLEXIBLE_HEADER_LAYOUT.size + len * size_of::<I>()
    }
}

impl<T: FlexibleArrayHeader, I> Object for FlexibleArray<I, T> {
    const OBJECT_ALIGN: usize = Self::FLEXIBLE_HEADER_LAYOUT.align;
    #[inline(always)]
    fn object_size(&self) -> usize {
        Self::flexible_size(self.header.len())
    }
    unsafe fn object_drop(&mut self) {
        drop_in_place(self.items_mut());
        drop_in_place(self);
    }
}

#[cfg(test)]
mod test {
    use core::{
        fmt::Debug,
        marker::PhantomData,
        mem::{align_of, forget, size_of},
        sync::atomic::{AtomicUsize, Ordering},
    };

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        common::ref_mut::RefMut,
        mem::{field_layout::FieldLayout, object::Object},
    };

    use super::{FlexibleArray, FlexibleArrayHeader};

    struct X<H, I>(H, PhantomData<I>);

    impl<H: Into<usize> + Copy, I> FlexibleArrayHeader for X<H, I> {
        // type Item = I;
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
        let v = ptr(&mut y) as *mut FlexibleArray<u8, X<u16, u8>>;
        unsafe {
            assert_eq!((*v).header.len(), 5);
            assert_eq!((*v).object_size(), 7);
            let items = (*v).items_mut();
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
        let v = ptr(&mut y) as *mut FlexibleArray<u32, X<u16, u32>>;
        unsafe {
            assert_eq!((*v).header.len(), 3);
            assert_eq!((*v).object_size(), 16);
            let items = (*v).items_mut();
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
        let v = ptr(&mut y) as *mut FlexibleArray<I, X<H, I>>;
        unsafe {
            assert_eq!((*v).header.len(), N);
            assert_eq!((*v).object_size(), size);
            assert_eq!(&*(*v).items_mut(), &old[..]);
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

    impl FlexibleArrayHeader for DropCount {
        // type Item = DropCount;
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
            let v = unsafe { x.to_mut_ptr() as *mut FlexibleArray<DropCount, DropCount> };
            unsafe {
                assert_eq!((*v).header.len(), 3);
                assert_eq!((*v).object_size(), size_of::<DropCountX>());
                assert_eq!((*v).object_size(), size_of::<DropCount>() * 4);
                let a = &*(*v).items_mut();
                assert_eq!(a.len(), 3);
                assert_eq!(a[0].0, &i as *const AtomicUsize);
                assert_eq!(a[1].0, &i as *const AtomicUsize);
                assert_eq!(a[2].0, &i as *const AtomicUsize);
            }
            assert_eq!(i.load(Ordering::Relaxed), 0);
            unsafe { (*v).object_drop() };
            assert_eq!(i.load(Ordering::Relaxed), 4);
            forget(x);
        }
        assert_eq!(i.load(Ordering::Relaxed), 4);
    }

    #[repr(C)]
    struct StaticVariable<T: FlexibleArrayHeader, I, const L: usize> {
        header: T,
        items: [I; L],
    }

    struct E();

    impl FlexibleArrayHeader for E {
        // type Item = u64;
        fn len(&self) -> usize {
            3
        }
    }

    const _: () = assert!(size_of::<StaticVariable<E, u64, 3>>() == 24);

    const _: () = assert!(FlexibleArray::<u64, E>::OBJECT_ALIGN == 8);
    const _: () = assert!(FlexibleArray::<u64, E>::FLEXIBLE_HEADER_LAYOUT.align == 8);
    const _: () = assert!(size_of::<E>() == 0);
    const _FL: FieldLayout<E, u64> = FieldLayout::align_to(align_of::<u64>());
    const _: () = assert!(_FL.align == 8);
    const _: () = assert!(_FL.size == 0);
    const _: () = assert!(FlexibleArray::<u64, E>::FLEXIBLE_HEADER_LAYOUT.size == 0);

    #[test]
    #[wasm_bindgen_test]
    fn empty_header_test() {
        static mut I: u8 = 0;
        unsafe { I = 0 };
        struct EmptyHeader();
        impl Drop for EmptyHeader {
            fn drop(&mut self) {
                unsafe { I += 1 };
            }
        }
        impl FlexibleArrayHeader for EmptyHeader {
            // type Item = u64;
            fn len(&self) -> usize {
                3
            }
        }
        let items: [u64; 3] = [0x1234567890abcdef, 0x1234567890abcdef, 0x1234567890abcdef];
        {
            let mut x = StaticVariable::<EmptyHeader, u64, 3> {
                header: EmptyHeader(),
                items,
            };
            let y = unsafe { &mut *(x.to_mut_ptr() as *mut FlexibleArray<u64, EmptyHeader>) };
            assert_eq!(size_of::<StaticVariable<EmptyHeader, u64, 3>>(), 24);
            assert_eq!(size_of::<EmptyHeader>(), 0);
            assert_eq!(y.object_size(), 24);
            assert_eq!(
                y.items_mut(),
                &[0x1234567890abcdef, 0x1234567890abcdef, 0x1234567890abcdef]
            );
            assert_eq!(unsafe { I }, 0);
            unsafe { y.object_drop() };
            assert_eq!(unsafe { I }, 1);
            forget(x)
        }
        assert_eq!(unsafe { I }, 1);
    }
}

use crate::{common::ref_mut::RefMut, mem::constructor::Constructor};

use super::{header::FlexibleArrayHeader, FlexibleArray};

pub struct FlexibleArrayConstructor<H: FlexibleArrayHeader, I: Iterator> {
    header: H,
    items: I,
}

impl<H: FlexibleArrayHeader, I: Iterator> FlexibleArrayConstructor<H, I> {
    pub fn new(header: H, items: I) -> Self {
        Self { header, items }
    }
}

impl<H: FlexibleArrayHeader, I: Iterator> Constructor for FlexibleArrayConstructor<H, I> {
    type Result = FlexibleArray<I::Item, H>;
    #[inline(always)]
    fn result_size(&self) -> usize {
        Self::Result::flexible_size(self.header.len())
    }
    unsafe fn construct(self, p: *mut Self::Result) {
        let v = &mut *p;
        v.header.to_mut_ptr().write(self.header);
        let mut src = self.items;
        for dst in v.items_mut() {
            dst.to_mut_ptr().write(src.next().unwrap());
        }
    }
}

impl<I: ExactSizeIterator> From<I> for FlexibleArrayConstructor<usize, I> {
    #[inline(always)]
    fn from(items: I) -> Self {
        items.len().constructor(items)
    }
}

#[cfg(test)]
mod test {
    use core::{mem::size_of, ptr::null_mut};

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        common::ref_mut::RefMut,
        mem::{
            constructor::Constructor, flexible_array::header::FlexibleArrayHeader, object::Object,
        },
    };

    use super::FlexibleArrayConstructor;

    #[repr(C)]
    struct StaticVariable<T: FlexibleArrayHeader, I, const L: usize> {
        header: T,
        items: [I; L],
    }

    fn gen_test(t: usize) {
        struct Header(u8, *mut u8);
        impl Drop for Header {
            fn drop(&mut self) {
                unsafe {
                    *self.1 += 1;
                }
            }
        }
        impl FlexibleArrayHeader for Header {
            fn len(&self) -> usize {
                self.0 as usize
            }
        }
        let mut i = 0;
        {
            let new = FlexibleArrayConstructor {
                header: Header(5, &mut i),
                items: [42u8, 43, 44, 45, 46, 47, 48].into_iter().take(t),
            };
            {
                let mut mem = StaticVariable::<Header, u8, 5> {
                    header: Header(0, null_mut()),
                    items: [0; 5],
                };
                let v = unsafe { (&mut mem).to_mut_ptr() as *mut _ };
                unsafe { new.construct(v) };
                let r = unsafe { &mut *v };
                assert_eq!(mem.header.len(), 5);
                assert_eq!(r.header.len(), 5);
                assert_eq!(mem.header.0, 5);
                assert_eq!(r.header.0, 5);
                assert_eq!(mem.header.1, unsafe { (&mut i).to_mut_ptr() });
                assert_eq!(r.header.1, unsafe { (&mut i).to_mut_ptr() });
                assert_eq!(r.object_size(), size_of::<usize>() * 2 + 5);
                assert_eq!(mem.items, [42, 43, 44, 45, 46]);
                assert_eq!(r.items_mut(), &[42, 43, 44, 45, 46]);
                assert_eq!(i, 0);
                unsafe { (*v).object_drop() };
                assert_eq!(i, 1);
            }
            assert_eq!(i, 2);
        }
        assert_eq!(i, 2);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_5() {
        gen_test(5);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_10() {
        gen_test(10);
    }

    #[test]
    #[should_panic]
    #[wasm_bindgen_test]
    fn test_2() {
        gen_test(2);
    }
}

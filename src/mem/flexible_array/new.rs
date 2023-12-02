use super::{
    super::{super::common::ref_mut::RefMut, new_in_place::NewInPlace},
    header::FlexibleArrayHeader,
    FlexibleArray,
};

struct FlexibleArrayNew<H: FlexibleArrayHeader, I: Iterator<Item = H::Item>> {
    header: H,
    items: I,
}

impl<H: FlexibleArrayHeader, I: Iterator<Item = H::Item>> NewInPlace for FlexibleArrayNew<H, I> {
    type Result = FlexibleArray<H>;
    fn result_size(&self) -> usize {
        Self::Result::flexible_array_size(self.header.len())
    }
    unsafe fn new_in_place(self, p: *mut Self::Result) {
        let v = &mut *p;
        v.header.as_mut_ptr().write(self.header);
        let mut src = self.items;
        for dst in v.get_items_mut() {
            dst.as_mut_ptr().write(src.next().unwrap());
        }
    }
}

#[cfg(test)]
mod test {
    use core::{mem::size_of, ptr::null_mut};

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{common::ref_mut::RefMut, mem::object::Object};

    use super::{
        super::{super::NewInPlace, FlexibleArrayHeader},
        FlexibleArrayNew,
    };

    #[repr(C)]
    struct StaticVariable<T: FlexibleArrayHeader, const L: usize> {
        header: T,
        items: [T::Item; L],
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
            type Item = u8;
            fn len(&self) -> usize {
                self.0 as usize
            }
        }
        let mut i = 0;
        {
            let new = FlexibleArrayNew {
                header: Header(5, &mut i),
                items: [42, 43, 44, 45, 46, 47, 48].into_iter().take(t),
            };
            {
                let mut mem = StaticVariable::<Header, 5> {
                    header: Header(0, null_mut()),
                    items: [0; 5],
                };
                let v = unsafe { (&mut mem).as_mut_ptr() as *mut _ };
                unsafe { new.new_in_place(v) };
                let r = unsafe { &mut *v };
                assert_eq!(mem.header.len(), 5);
                assert_eq!(r.header.len(), 5);
                assert_eq!(mem.header.0, 5);
                assert_eq!(r.header.0, 5);
                assert_eq!(mem.header.1, unsafe { (&mut i).as_mut_ptr() });
                assert_eq!(r.header.1, unsafe { (&mut i).as_mut_ptr() });
                assert_eq!(r.object_size(), size_of::<usize>() * 2 + 5);
                assert_eq!(mem.items, [42, 43, 44, 45, 46]);
                assert_eq!(r.get_items_mut(), &[42, 43, 44, 45, 46]);
                assert_eq!(i, 0);
                unsafe { (*v).object_drop_in_place() };
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

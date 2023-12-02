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
    // optional
    const LAYOUT: FieldLayout<Self, Self::Item> = FieldLayout::align_to(align_of::<Self::Item>());
    fn get_items_mut(&mut self) -> &mut [Self::Item] {
        unsafe { from_raw_parts_mut(Self::LAYOUT.to_adjacent(self), self.len()) }
    }
}

#[repr(transparent)]
pub struct Variable<T: VariableHeader>(T);

impl<T: VariableHeader> Variable<T> {
    const LAYOUT: FieldLayout<T, T::Item> = FieldLayout::align_to(align_of::<T::Item>());
    pub fn get_items_mut(&mut self) -> &mut [T::Item] {
        self.0.get_items_mut()
    }
}

impl<T: VariableHeader> Object for Variable<T> {
    const OBJECT_ALIGN: usize = T::LAYOUT.align;
    fn object_size(&self) -> usize {
        T::LAYOUT.size + self.0.len() * size_of::<T::Item>()
    }
    unsafe fn object_drop_in_place(&mut self) {
        drop_in_place(self.0.get_items_mut());
        drop_in_place(self);
    }
}

#[cfg(test)]
mod test {
    use core::marker::PhantomData;

    use wasm_bindgen_test::wasm_bindgen_test;

    use super::VariableHeader;

    struct X<I>(usize, PhantomData<I>);

    impl<I> VariableHeader for X<I> {
        type Item = I;
        fn len(&self) -> usize {
            self.0
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_u8_5() {
        #[repr(C)]
        struct X_8_5 {
            len: usize,
            items: [u8; 5],
        }
        let v = X_8_5 {
            len: 5,
            items: [42, 43, 44, 45, 46],
        };
        let x = &v as *const X_8_5 as *const X<u8>;
        unsafe {
            assert_eq!((*x).len(), 5);
        }
    }
}

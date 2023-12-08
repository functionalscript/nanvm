pub mod header;

use core::{alloc::Layout, marker::PhantomData};

use super::{field_layout::FieldLayout, manager::Dealloc, object::Object};

#[repr(transparent)]
pub struct Block<T: Object, D: Dealloc> {
    pub header: D::BlockHeader,
    _0: PhantomData<T>,
}

impl<T: Object, D: Dealloc> Block<T, D> {
    const BLOCK_HEADER_LAYOUT: FieldLayout<D::BlockHeader, T> =
        FieldLayout::align_to(T::OBJECT_ALIGN);
    #[inline(always)]
    pub const fn block_layout(object_size: usize) -> Layout {
        Self::BLOCK_HEADER_LAYOUT.layout(object_size)
    }
    pub unsafe fn delete(&mut self) {
        let object = self.object_mut();
        let object_size = object.object_size();
        object.object_drop_in_place();
        D::dealloc(self as *mut _ as *mut u8, Self::block_layout(object_size));
    }
    #[inline(always)]
    pub fn object(&self) -> &T {
        unsafe { &*Self::BLOCK_HEADER_LAYOUT.to_adjacent(&self.header) }
    }
    #[inline(always)]
    pub fn object_mut(&mut self) -> &mut T {
        unsafe { &mut *Self::BLOCK_HEADER_LAYOUT.to_adjacent_mut(&mut self.header) }
    }
}

#[cfg(test)]
mod test {
    use core::alloc::Layout;

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{
        block::Block, fixed::Fixed, manager::Dealloc, object::Object, ref_::counter_update::RefCounterUpdate,
    };

    use super::header::BlockHeader;

    struct M();

    impl Dealloc for M {
        type BlockHeader = BH;
        unsafe fn dealloc(_: *mut u8, _: Layout) {}
    }

    #[derive(Default)]
    struct BH();

    impl BlockHeader for BH {
        unsafe fn ref_counter_update(&self, _: RefCounterUpdate) -> isize {
            todo!()
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_0() {
        assert_eq!(Block::<Fixed<()>, M>::block_layout(0).size(), 0);
        assert_eq!(Block::<Fixed<()>, M>::block_layout(0).align(), 1);
        assert_eq!(Block::<Fixed<()>, M>::block_layout(2).size(), 2);
        assert_eq!(Block::<Fixed<()>, M>::block_layout(2).align(), 1);
        let mut b = Block::<Fixed<()>, M> {
            header: BH::default(),
            _0: Default::default(),
        };
        assert_eq!(b.object().object_size(), 0);
        let x = b.object_mut();
        assert_eq!(x.object_size(), 0);
        unsafe { b.delete() };
    }
}

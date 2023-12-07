pub mod header;

use core::{alloc::Layout, marker::PhantomData};

use self::header::BlockHeader;

use super::{field_layout::FieldLayout, manager::Manager, object::Object};

#[repr(transparent)]
pub struct Block<BH: BlockHeader, T: Object> {
    pub header: BH,
    _0: PhantomData<T>,
}

impl<BH: BlockHeader, T: Object> Block<BH, T> {
    const BLOCK_HEADER_LAYOUT: FieldLayout<BH, T> = FieldLayout::align_to(T::OBJECT_ALIGN);
    #[inline(always)]
    pub const fn block_layout(object_size: usize) -> Layout {
        unsafe {
            Layout::from_size_align_unchecked(
                Self::BLOCK_HEADER_LAYOUT.size + object_size,
                Self::BLOCK_HEADER_LAYOUT.align,
            )
        }
    }
    pub unsafe fn delete(&mut self) {
        let object = self.object_mut();
        let object_size = object.object_size();
        object.object_drop_in_place();
        <BH::Manager as Manager>::dealloc(
            self as *mut _ as *mut u8,
            Self::block_layout(object_size),
        );
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
        block::Block, fixed::Fixed, manager::Manager, object::Object, ref_::update::RefUpdate,
    };

    use super::header::BlockHeader;

    struct M();

    impl Manager for M {
        type BlockHeader = BH;
        unsafe fn alloc(self, layout: Layout) -> *mut u8 {
            todo!()
        }
        unsafe fn dealloc(ptr: *mut u8, layout: Layout) {}
    }

    #[derive(Default)]
    struct BH();

    impl BlockHeader for BH {
        type Manager = M;
        unsafe fn ref_update(&mut self, _: RefUpdate) -> isize {
            todo!()
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        assert_eq!(Block::<BH, Fixed<()>>::block_layout(0).size(), 0);
        assert_eq!(Block::<BH, Fixed<()>>::block_layout(0).align(), 1);
        assert_eq!(Block::<BH, Fixed<()>>::block_layout(2).size(), 2);
        assert_eq!(Block::<BH, Fixed<()>>::block_layout(2).align(), 1);
        let b = Block::<BH, Fixed<()>> {
            header: BH::default(),
            _0: Default::default(),
        };
        assert_eq!(b.object().object_size(), 0);
    }
}

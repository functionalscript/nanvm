use core::sync::atomic::{AtomicIsize, Ordering};

use std::alloc::{alloc, dealloc};

use crate::common::ref_mut::RefMut;

use super::{
    block::{header::BlockHeader, Block},
    new_in_place::NewInPlace,
    object::Object,
    ref_::{update::RefUpdate, Ref},
    Manager,
};

pub struct Global();

pub struct GlobalHeader(AtomicIsize);

impl Default for GlobalHeader {
    fn default() -> Self {
        Self(AtomicIsize::new(1))
    }
}

impl BlockHeader for GlobalHeader {
    #[inline(always)]
    unsafe fn ref_update(&self, i: RefUpdate) -> isize {
        self.0.fetch_add(i as isize, Ordering::Relaxed)
    }
    unsafe fn delete<T: Object>(block: &mut Block<Self, T>) {
        let object = block.object();
        let object_size = object.object_size();
        object.object_drop_in_place();
        dealloc(
            block as *mut _ as *mut u8,
            Block::<Self, T>::block_layout(object_size),
        );
    }
}

impl Manager for Global {
    type BlockHeader = GlobalHeader;
    unsafe fn new<N: NewInPlace>(self, new_in_place: N) -> Ref<N::Result, Self> {
        let p = alloc(Block::<GlobalHeader, N::Result>::block_layout(
            new_in_place.result_size(),
        )) as *mut Block<GlobalHeader, _>;
        let block = &mut *p;
        block.header.as_mut_ptr().write(GlobalHeader::default());
        new_in_place.new_in_place(block.object());
        Ref::new(p)
    }
}

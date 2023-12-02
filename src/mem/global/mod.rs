use std::alloc::alloc;

use crate::common::ref_mut::RefMut;

use self::header::GlobalHeader;

use super::{block::Block, new_in_place::NewInPlace, ref_::Ref, Manager};

mod header;

pub struct Global();

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

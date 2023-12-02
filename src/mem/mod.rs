mod block;
mod field_layout;
mod fixed;
mod flexible_array;
mod new_in_place;
mod object;
mod ref_;

use core::{
    ptr::drop_in_place,
    sync::atomic::{AtomicIsize, Ordering},
};
use std::alloc::{alloc, dealloc};

use self::{
    block::{header::BlockHeader, Block},
    new_in_place::NewInPlace,
    object::Object,
    ref_::{update::RefUpdate, Ref},
};

/// Block = (Header, Object)
pub trait Manager: Sized {
    type BlockHeader: BlockHeader;
    /// Allocate a block of memory for a new T object and initialize the object with the `new_in_place`.
    unsafe fn new<N: NewInPlace>(self, new_in_place: N) -> Ref<N::Result, Self>;
}

struct Global();

struct GlobalHeader(AtomicIsize);

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
        let block = alloc(Block::<GlobalHeader, N::Result>::block_layout(
            new_in_place.result_size(),
        )) as *mut Block<_, _>;
        (*block).header = GlobalHeader(AtomicIsize::new(1));
        new_in_place.new_in_place((*block).object());
        Ref::new(block)
    }
}

#[cfg(test)]
mod test {
    use std::mem::{align_of, size_of};

    struct MyStruct {
        a: u8,  // 1 byte
        b: u16, // 2 bytes
        c: u8,  // 1 byte
        d: u8,
    }

    const _: () = assert!(size_of::<MyStruct>() == 6);
    const _: () = assert!(align_of::<MyStruct>() == 2);
}

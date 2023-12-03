use core::alloc::Layout;
use std::alloc::{alloc, dealloc};

use self::header::GlobalHeader;

use super::Manager;

mod header;

pub struct Global();

impl Manager for Global {
    type BlockHeader = GlobalHeader;
    unsafe fn alloc(self, layout: Layout) -> *mut u8 {
        alloc(layout)
    }
    unsafe fn dealloc(ptr: *mut u8, layout: Layout) {
        dealloc(ptr, layout)
    }
}

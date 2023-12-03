use core::alloc::Layout;
use std::alloc::alloc;

use self::header::GlobalHeader;

use super::Manager;

mod header;

pub struct Global();

impl Manager for Global {
    type BlockHeader = GlobalHeader;
    unsafe fn alloc(self, layout: Layout) -> *mut u8 {
        alloc(layout)
    }
}

use super::{
    block::{header::BlockHeader, Block},
    manager::Dealloc,
    object::Object,
    ref_::counter_update::RefCounterUpdate,
};

pub trait Variant: Copy {
    type BlockHeader: BlockHeader;
    fn get_block_header(self) -> Option<*const Self::BlockHeader>;
    unsafe fn delete(self, block: *mut Self::BlockHeader);
    unsafe fn ref_counter_update(self, i: RefCounterUpdate) -> Option<*mut Self::BlockHeader> {
        if let Some(header) = self.get_block_header() {
            if (*header).ref_counter_update(i) == 0 {
                Some(header as *const _ as *mut _)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<T: Object, D: Dealloc> Variant for *const Block<T, D> {
    type BlockHeader = D::BlockHeader;
    fn get_block_header(self) -> Option<*const Self::BlockHeader> {
        unsafe { Some(&(*self).header) }
    }
    unsafe fn delete(self, block: *mut Self::BlockHeader) {
        (*(block as *mut Block<T, D>)).delete();
    }
}

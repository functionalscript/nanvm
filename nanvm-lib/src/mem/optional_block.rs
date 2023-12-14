use super::{
    block::{header::BlockHeader, Block},
    manager::Dealloc,
    object::Object,
    ref_counter_update::RefCounterUpdate,
};

pub trait OptionalBlock: Copy {
    type BlockHeader: BlockHeader;
    fn try_get_block_header(self) -> Option<*const Self::BlockHeader>;
    unsafe fn delete(self, block_header: *mut Self::BlockHeader);
    //
    unsafe fn ref_counter_update(self, i: RefCounterUpdate) -> Option<*mut Self::BlockHeader> {
        match self.try_get_block_header() {
            Some(header) if (*header).ref_counter_update(i) == 0 => Some(header as *const _ as _),
            _ => None,
        }
    }
}

impl<T: Object, D: Dealloc> OptionalBlock for *const Block<T, D> {
    type BlockHeader = D::BlockHeader;
    fn try_get_block_header(self) -> Option<*const Self::BlockHeader> {
        unsafe { Some(&(*self).header) }
    }
    unsafe fn delete(self, block_header: *mut Self::BlockHeader) {
        (*(block_header as *mut Block<T, D>)).delete();
    }
}

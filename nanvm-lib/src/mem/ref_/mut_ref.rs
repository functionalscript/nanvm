use crate::mem::{
    block::{header::BlockHeader, Block},
    manager::Manager,
    object::Object,
    ref_::update::RefUpdate,
};

/// A reference to a mutable object allocated by a memory manager.
#[repr(transparent)]
pub struct MutRef<T: Object, M: Manager>(*mut Block<M::BlockHeader, T>);

impl<T: Object, M: Manager> Drop for MutRef<T, M> {
    fn drop(&mut self) {
        unsafe {
            let p = &mut *self.0;
            assert_eq!(p.header.ref_update(RefUpdate::Read), 1);
            p.delete();
        }
    }
}

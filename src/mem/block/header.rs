use crate::{
    common::ref_mut::RefMut,
    mem::{object::Object, ref_::update::RefUpdate},
};

use super::Block;

pub trait BlockHeader: Sized {
    // required
    unsafe fn ref_update(&self, i: RefUpdate) -> isize;
    unsafe fn delete<T: Object>(block: &mut Block<Self, T>);
    //
    #[inline(always)]
    unsafe fn block<T: Object>(&mut self) -> &mut Block<Self, T> {
        &mut *(self.as_mut_ptr() as *mut _)
    }
}

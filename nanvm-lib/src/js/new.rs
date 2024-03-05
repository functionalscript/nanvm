use crate::mem::manager::Manager;

use super::{any::Any, any_cast::AnyCast, js_array::new_array, js_object::new_object, js_string::{new_string, JsStringRef}};

pub trait New: Manager {
    fn new_js_array(
        self,
        i: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = Any<Self::Dealloc>>>,
    ) -> Any<Self::Dealloc> {
        new_array(self, i).to_ref().move_to_any()
    }
    fn new_js_string(
        self,
        s: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = u16>>,
    ) -> Any<Self::Dealloc> {
        new_string(self, s).to_ref().move_to_any()
    }
    fn new_js_object(
        self,
        i: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = (JsStringRef<Self::Dealloc>, Any<Self::Dealloc>)>>,
    ) -> Any<Self::Dealloc> {
        new_object(self, i).to_ref().move_to_any()
    }
}

impl<M: Manager> New for M {}

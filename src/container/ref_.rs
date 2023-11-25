use super::{Base, Update};

trait ToBase: Clone {
    fn get_base(&self) -> Option<&mut Base>;
    fn dealloc(base: &mut Base);
}

#[repr(transparent)]
struct Ref<T: ToBase>(T);

impl<T: ToBase> Ref<T> {
    #[inline(always)]
    fn new(t: T) -> Self {
        Self(t)
    }
}

impl<T: ToBase> Clone for Ref<T> {
    fn clone(&self) -> Self {
        let result = Self::new(self.0.clone());
        if let Some(base) = result.0.get_base() {
            unsafe { base.update(Update::AddRef) };
        }
        result
    }
}

impl<T: ToBase> Drop for Ref<T> {
    fn drop(&mut self) {
        if let Some(base) = self.0.get_base() {
            if unsafe { base.update(Update::AddRef) } != 0 {
                return;
            }
            T::dealloc(base);
        }
    }
}

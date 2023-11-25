use std::mem::forget;

use crate::{
    container::{Container, ContainerRef, Info, Ref},
    ptr_subset::{PtrSubset, PTR_SUBSET_SUPERPOSITION},
    type_::Type,
    value::{number, string::StringHeader},
};

use super::{
    extension::{BOOL, EXTENSION, FALSE, OBJECT, PTR, STRING},
    internal::Internal,
    object::{ObjectHeader, ObjectRef},
    string::StringRef,
};

pub type Unknown = Ref<Internal>;

impl From<f64> for Unknown {
    fn from(n: f64) -> Self {
        let n = n.to_bits();
        assert!(number::is_valid(n));
        Self::from_u64(n)
    }
}

impl Unknown {
    #[inline(always)]
    const fn from_u64(u: u64) -> Self {
        Self::from_raw(Internal(u))
    }
    #[inline(always)]
    const fn u64(&self) -> u64 {
        self.get().0
    }
    // number
    #[inline(always)]
    const fn is_number(&self) -> bool {
        !EXTENSION.has(self.u64())
    }
    fn get_number(&self) -> Option<f64> {
        if self.is_number() {
            return Some(f64::from_bits(self.u64()));
        }
        None
    }
    // bool
    #[inline(always)]
    const fn from_bool(b: bool) -> Self {
        Self::from_u64((b as u64) | BOOL.tag)
    }
    #[inline(always)]
    const fn is_bool(&self) -> bool {
        BOOL.has(self.u64())
    }
    const fn get_bool(&self) -> Option<bool> {
        if self.is_bool() {
            return Some(self.u64() != FALSE);
        }
        None
    }
    //
    #[inline(always)]
    const fn is_ptr(&self) -> bool {
        PTR.has(self.u64())
    }
    //
    #[inline(always)]
    const fn null() -> Self {
        Self::from_u64(OBJECT.subset().tag)
    }
    #[inline(always)]
    const fn is_null(&self) -> bool {
        self.u64() == OBJECT.subset().tag
    }
    //
    #[inline(always)]
    fn from_ref<T: Info>(ps: PtrSubset<T>, s: Ref<*mut Container<T>>) -> Self {
        let p: *mut Container<T> = *s.get();
        forget(s);
        Self::from_u64((p as u64) | ps.subset().tag)
    }
    fn get_container_ptr<T: Info>(&self, ps: &PtrSubset<T>) -> Option<*mut Container<T>> {
        let v = self.u64();
        if ps.subset().has(v) {
            let p = v & PTR_SUBSET_SUPERPOSITION;
            if p == 0 {
                return None;
            }
            return Some(p as *mut Container<T>);
        }
        None
    }
    fn get_container<T: Info>(&self, ps: &PtrSubset<T>) -> Option<&mut Container<T>> {
        if let Some(p) = self.get_container_ptr(ps) {
            return Some(unsafe { &mut *p });
        }
        None
    }
    fn get_container_ref<T: Info>(self, ps: &PtrSubset<T>) -> Option<ContainerRef<T>> {
        if let Some(c) = self.get_container_ptr(ps) {
            forget(self);
            return Some(ContainerRef::from_raw(c));
        }
        None
    }
    // string
    #[inline(always)]
    const fn is_string(&self) -> bool {
        STRING.subset().has(self.u64())
    }
    #[inline(always)]
    fn from_string(s: StringRef) -> Self {
        Self::from_ref(STRING, s)
    }
    #[inline(always)]
    fn get_string(&self) -> Option<&mut Container<StringHeader>> {
        self.get_container(&STRING)
    }
    #[inline(always)]
    fn get_string_ref(self) -> Option<StringRef> {
        self.get_container_ref(&STRING)
    }
    // object
    #[inline(always)]
    const fn is_object(&self) -> bool {
        OBJECT.subset().has(self.u64())
    }
    #[inline(always)]
    fn from_object(s: ObjectRef) -> Self {
        Self::from_ref(OBJECT, s)
    }
    #[inline(always)]
    fn get_object(&self) -> Option<&mut Container<ObjectHeader>> {
        self.get_container(&OBJECT)
    }
    #[inline(always)]
    fn get_object_ref(self) -> Option<ObjectRef> {
        self.get_container_ref(&OBJECT)
    }
    //
    const fn get_type(&self) -> Type {
        if self.is_ptr() {
            if self.is_string() {
                Type::String
            } else {
                Type::Object
            }
        } else {
            if self.is_number() {
                Type::Number
            } else {
                Type::Bool
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use wasm_bindgen_test::wasm_bindgen_test;

    use super::{super::extension::TRUE, number::NAN, *};

    const _: () = assert!(BOOL.has(FALSE));
    const _: () = assert!(BOOL.has(TRUE));
    const _: () = assert!(!BOOL.has(0));
    const _: () = assert!(!BOOL.has(NAN));
    const _: () = assert!(BOOL.has(EXTENSION.mask));

    #[test]
    #[wasm_bindgen_test]
    fn test_unsized() {
        let _x: Rc<[u8]> = Rc::new([1, 3]);
        // let _y: Rc<(u8, [u8])> = Rc::new((5, [1, 3]));
        // let r = Vec::default();
        // let n = 4 + 4;
        // let _y: Rc<[u8]> = Rc::new([5; n]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_number() {
        assert_eq!(Unknown::from(1.0).get_number(), Some(1.0));
        assert_eq!(Unknown::from(-1.0).get_number(), Some(-1.0));
        assert_eq!(
            Unknown::from(f64::INFINITY).get_number(),
            Some(f64::INFINITY)
        );
        assert_eq!(
            Unknown::from(f64::NEG_INFINITY).get_number(),
            Some(f64::NEG_INFINITY)
        );
        assert!(Unknown::from(f64::NAN).get_number().unwrap().is_nan());
        //
        assert_eq!(Unknown::from_bool(true).get_number(), None);
        assert_eq!(Unknown::null().get_number(), None);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_bool() {
        assert_eq!(Unknown::from_bool(true).get_bool(), Some(true));
        assert_eq!(Unknown::from_bool(false).get_bool(), Some(false));
        //
        assert_eq!(Unknown::from(15.0).get_bool(), None);
        assert_eq!(Unknown::null().get_bool(), None);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_null() {
        assert!(Unknown::null().is_null());
        //
        assert!(!Unknown::from(-15.7).is_null());
        assert!(!Unknown::from_bool(false).is_null());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_object() {
        assert!(Unknown::null().is_object());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_type() {
        assert_eq!(Unknown::from(15.0).get_type(), Type::Number);
        assert_eq!(Unknown::from_bool(true).get_type(), Type::Bool);
        assert_eq!(Unknown::null().get_type(), Type::Object);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_string() {
        let mut s = StringRef::alloc(StringHeader(), [].into_iter());
        assert!(Unknown::from_string(s.clone()).is_string());
        let v = s.get_items_mut();
        assert!(v.is_empty());
        //
        assert!(!Unknown::from(15.0).is_string());
        assert!(!Unknown::from_bool(true).is_string());
        assert!(!Unknown::null().is_string());
    }
}

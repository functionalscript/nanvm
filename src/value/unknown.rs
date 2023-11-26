use std::{mem::forget, result};

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
    string::{StringContainer, StringRef},
};

pub type Unknown = Ref<Internal>;

type Result<T> = result::Result<T, ()>;

impl From<f64> for Unknown {
    #[inline(always)]
    fn from(n: f64) -> Self {
        let n = n.to_bits();
        assert!(number::is_valid(n));
        Self::from_u64(n)
    }
}

impl TryFrom<Unknown> for f64 {
    type Error = ();
    fn try_from(u: Unknown) -> Result<Self> {
        if u.is_number() {
            return Ok(f64::from_bits(u.u64()));
        }
        Err(())
    }
}

impl From<bool> for Unknown {
    #[inline(always)]
    fn from(b: bool) -> Self {
        Self::from_bool(b)
    }
}

impl TryFrom<Unknown> for bool {
    type Error = ();
    #[inline(always)]
    fn try_from(u: Unknown) -> Result<Self> {
        u.get_bool()
    }
}

impl From<StringRef> for Unknown {
    #[inline(always)]
    fn from(s: StringRef) -> Self {
        Self::from_ref(STRING, s)
    }
}

impl<'a> TryFrom<&'a Unknown> for &'a mut StringContainer {
    type Error = ();
    #[inline(always)]
    fn try_from(u: &'a Unknown) -> Result<Self> {
        u.get_container(&STRING)
    }
}

impl TryFrom<Unknown> for StringRef {
    type Error = ();
    #[inline(always)]
    fn try_from(u: Unknown) -> Result<Self> {
        u.get_container_ref(&STRING)
    }
}

impl From<ObjectRef> for Unknown {
    #[inline(always)]
    fn from(o: ObjectRef) -> Self {
        Self::from_ref(OBJECT, o)
    }
}

impl TryFrom<Unknown> for ObjectRef {
    type Error = ();
    #[inline(always)]
    fn try_from(u: Unknown) -> Result<Self> {
        u.get_container_ref(&OBJECT)
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
    // bool
    #[inline(always)]
    const fn from_bool(b: bool) -> Self {
        Self::from_u64((b as u64) | BOOL.tag)
    }
    #[inline(always)]
    const fn is_bool(&self) -> bool {
        BOOL.has(self.u64())
    }
    const fn get_bool(&self) -> Result<bool> {
        if self.is_bool() {
            return Ok(self.u64() != FALSE);
        }
        Err(())
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
    fn from_ref<T: Info>(ps: PtrSubset<T>, s: ContainerRef<T>) -> Self {
        let p: *mut Container<T> = *s.get();
        forget(s);
        Self::from_u64((p as u64) | ps.subset().tag)
    }
    fn get_container_ptr<T: Info>(&self, ps: &PtrSubset<T>) -> Result<*mut Container<T>> {
        let v = self.u64();
        if ps.subset().has(v) {
            let p = v & PTR_SUBSET_SUPERPOSITION;
            if p == 0 {
                return Err(());
            }
            return Ok(p as *mut Container<T>);
        }
        Err(())
    }
    fn get_container<T: Info>(&self, ps: &PtrSubset<T>) -> Result<&mut Container<T>> {
        if let Ok(p) = self.get_container_ptr(ps) {
            return Ok(unsafe { &mut *p });
        }
        Err(())
    }
    fn get_container_ref<T: Info>(self, ps: &PtrSubset<T>) -> Result<ContainerRef<T>> {
        if let Ok(c) = self.get_container_ptr(ps) {
            forget(self);
            return Ok(ContainerRef::from_raw(c));
        }
        Err(())
    }
    // string
    #[inline(always)]
    const fn is_string(&self) -> bool {
        STRING.subset().has(self.u64())
    }
    #[inline(always)]
    fn get_string(&self) -> Result<&mut Container<StringHeader>> {
        self.get_container(&STRING)
    }
    // object
    #[inline(always)]
    const fn is_object(&self) -> bool {
        OBJECT.subset().has(self.u64())
    }
    #[inline(always)]
    fn get_object(&self) -> Result<&mut Container<ObjectHeader>> {
        self.get_container(&OBJECT)
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
        assert_eq!(Unknown::from(1.0).try_into(), Ok(1.0));
        //let y = -1.0;
        let x: Unknown = (-1.0).into();
        assert_eq!(x.try_into(), Ok(-1.0));
        assert_eq!(Unknown::from(f64::INFINITY).try_into(), Ok(f64::INFINITY));
        assert_eq!(
            Unknown::from(f64::NEG_INFINITY).try_into(),
            Ok(f64::NEG_INFINITY)
        );
        assert!(f64::try_from(Unknown::from(f64::NAN)).unwrap().is_nan());
        //
        assert_eq!(f64::try_from(Unknown::from_bool(true)), Err(()));
        assert_eq!(f64::try_from(Unknown::null()), Err(()));
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_bool() {
        assert_eq!(Unknown::from_bool(true).get_bool(), Ok(true));
        assert_eq!(Unknown::from(false).get_bool(), Ok(false));
        //
        assert_eq!(Unknown::from(15.0).get_bool(), Err(()));
        assert_eq!(Unknown::null().get_bool(), Err(()));
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_null() {
        assert!(Unknown::null().is_null());
        //
        assert!(!Unknown::from(-15.7).is_null());
        assert!(!Unknown::from(false).is_null());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_type() {
        assert_eq!(Unknown::from(15.0).get_type(), Type::Number);
        assert_eq!(Unknown::from(true).get_type(), Type::Bool);
        assert_eq!(Unknown::null().get_type(), Type::Object);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_string() {
        let mut s = StringRef::alloc(StringHeader(), [].into_iter());
        assert!(Unknown::from(s.clone()).is_string());
        let v = s.get_items_mut();
        assert!(v.is_empty());
        //
        assert!(!Unknown::from(15.0).is_string());
        assert!(!Unknown::from(true).is_string());
        assert!(!Unknown::null().is_string());

        let s = StringRef::alloc(StringHeader(), [0x20, 0x21].into_iter());
        assert!(Unknown::from(s.clone()).is_string());
        let v = s.get_items_mut();
        assert_eq!(v, [0x20, 0x21]);
        let u = Unknown::from(s);
        {
            let s = <&mut StringContainer>::try_from(&u).unwrap();
            let items = s.get_items_mut();
            assert_eq!(items, [0x20, 0x21]);
        }
        let s = StringRef::try_from(u).unwrap();
        let items = s.get_items_mut();
        assert_eq!(items, [0x20, 0x21]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_object() {
        assert!(Unknown::null().is_object());

        let o = ObjectRef::alloc(ObjectHeader(), [].into_iter());
        assert!(Unknown::from(o.clone()).is_object());
        let v = o.get_items_mut();
        assert!(v.is_empty());
        //
        assert!(!Unknown::from(15.0).is_object());
        assert!(!Unknown::from(true).is_object());
    }
}

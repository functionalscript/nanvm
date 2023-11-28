use std::{mem::forget, result};

use crate::{
    container::{Container, ContainerRef, Info, Ref},
    ptr_subset::{PtrSubset, PTR_SUBSET_SUPERPOSITION},
};

use super::{
    cast::Cast,
    extension::{OBJECT, RC, STRING},
    internal::Internal,
    null::Null,
    object::{ObjectContainer, ObjectRef},
    string::{StringContainer, StringRef},
    type_::Type,
};

pub type Unknown = Ref<Internal>;

type Result<T> = result::Result<T, ()>;

impl<T: Cast> From<T> for Unknown {
    #[inline(always)]
    fn from(t: T) -> Self {
        unsafe { Self::from_u64(t.cast_into()) }
    }
}

impl<'a> TryFrom<&'a Unknown> for &'a mut StringContainer {
    type Error = ();
    #[inline(always)]
    fn try_from(u: &'a Unknown) -> Result<Self> {
        u.get_container(&STRING)
    }
}

impl<'a> TryFrom<&'a Unknown> for &'a mut ObjectContainer {
    type Error = ();
    #[inline(always)]
    fn try_from(u: &'a Unknown) -> Result<Self> {
        u.get_container(&OBJECT)
    }
}

impl Unknown {
    #[inline(always)]
    pub unsafe fn from_u64(u: u64) -> Self {
        Self::from_raw(Internal(u))
    }
    #[inline(always)]
    unsafe fn u64(&self) -> u64 {
        self.get().0
    }
    // generic
    #[inline(always)]
    fn is<T: Cast>(&self) -> bool {
        unsafe { T::cast_is(self.u64()) }
    }
    fn try_to<T: Cast>(self) -> Result<T> {
        if self.is::<T>() {
            return Ok(unsafe { T::cast_from(self.move_to_raw().0) });
        }
        Err(())
    }
    //
    #[inline(always)]
    fn is_rc(&self) -> bool {
        RC.has(unsafe { self.u64() })
    }
    //
    #[inline(always)]
    fn from_ref<T: Info>(ps: PtrSubset<T>, s: ContainerRef<T>) -> Self {
        let p: *mut Container<T> = *s.get();
        forget(s);
        unsafe { Self::from_u64((p as u64) | ps.subset().tag) }
    }
    fn get_container_ptr<T: Info>(&self, ps: &PtrSubset<T>) -> Result<*mut Container<T>> {
        let v = unsafe { self.u64() };
        if ps.subset().has(v) {
            let p = v & PTR_SUBSET_SUPERPOSITION;
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
            return Ok(unsafe { ContainerRef::from_raw(c) });
        }
        Err(())
    }
    // object
    #[inline(always)]
    fn is_object(&self) -> bool {
        OBJECT.subset().has(unsafe { self.u64() })
    }
    //
    fn get_type(&self) -> Type {
        if self.is_rc() {
            if self.is::<StringRef>() {
                Type::String
            } else {
                Type::Object
            }
        } else {
            if self.is::<f64>() {
                Type::Number
            } else if self.is::<Null>() {
                Type::Null
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

    use crate::value::{
        extension::{BOOL, EXTENSION, FALSE},
        null::Null,
        object::ObjectHeader,
        string::StringHeader,
    };

    use super::{
        super::{extension::TRUE, number::NAN},
        *,
    };

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
        assert_eq!(Unknown::from(1.0).try_to(), Ok(1.0));
        //let y = -1.0;
        let x: Unknown = (-1.0).into();
        assert_eq!(x.try_to(), Ok(-1.0));
        assert_eq!(Unknown::from(f64::INFINITY).try_to(), Ok(f64::INFINITY));
        assert_eq!(
            Unknown::from(f64::NEG_INFINITY).try_to(),
            Ok(f64::NEG_INFINITY)
        );
        assert!(Unknown::from(f64::NAN).try_to::<f64>().unwrap().is_nan());
        //
        assert_eq!(Unknown::from(true).try_to::<f64>(), Err(()));
        assert_eq!(Unknown::from(Null()).try_to::<f64>(), Err(()));
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_bool() {
        assert_eq!(Unknown::from(true).try_to(), Ok(true));
        assert_eq!(Unknown::from(false).try_to(), Ok(false));
        //
        assert_eq!(Unknown::from(15.0).try_to::<bool>(), Err(()));
        assert_eq!(Unknown::from(Null()).try_to::<bool>(), Err(()));
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_null() {
        assert!(Unknown::from(Null()).is::<Null>());
        //
        assert!(!Unknown::from(-15.7).is::<Null>());
        assert!(!Unknown::from(false).is::<Null>());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_type() {
        assert_eq!(Unknown::from(15.0).get_type(), Type::Number);
        assert_eq!(Unknown::from(true).get_type(), Type::Bool);
        assert_eq!(Null().unknown().get_type(), Type::Null);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_string() {
        let s = StringRef::alloc(StringHeader(), [].into_iter());
        assert!(Unknown::from(s.clone()).is::<StringRef>());
        let v = s.get_items_mut();
        assert!(v.is_empty());
        //
        assert!(!Unknown::from(15.0).is::<StringRef>());
        assert!(!Unknown::from(true).is::<StringRef>());
        assert!(!Null().unknown().is::<StringRef>());

        let s = StringRef::alloc(StringHeader(), [0x20, 0x21].into_iter());
        assert!(Unknown::from(s.clone()).is::<StringRef>());
        let v = s.get_items_mut();
        assert_eq!(v, [0x20, 0x21]);
        let u = Unknown::from(s);
        {
            let s = <&mut StringContainer>::try_from(&u).unwrap();
            let items = s.get_items_mut();
            assert_eq!(items, [0x20, 0x21]);
        }
        let s = u.try_to::<StringRef>().unwrap();
        let items = s.get_items_mut();
        assert_eq!(items, [0x20, 0x21]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_object() {
        assert!(!Null().unknown().is_object());

        let o = ObjectRef::alloc(ObjectHeader(), [].into_iter());
        assert!(Unknown::from(o.clone()).is_object());
        let v = o.get_items_mut();
        assert!(v.is_empty());
        //
        assert!(!15.0.unknown().is_object());
        assert!(!true.unknown().is_object());

        let o = ObjectRef::alloc(ObjectHeader(), [].into_iter());
        let u = o.unknown();
        assert_eq!(u.get_type(), Type::Object);
        {
            let o = u.try_to::<ObjectRef>().unwrap();
            let items = o.get_items_mut();
            assert!(items.is_empty());
        }
    }
}

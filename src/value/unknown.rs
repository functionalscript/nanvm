use std::result;

use crate::container::{Container, OptionalRc};

use super::{
    cast::Cast,
    extension::{PTR_SUBSET_SUPERPOSITION, RC},
    internal::Internal,
    null::Null,
    string::StringRc,
    tag_rc::TagRc,
    type_::Type,
};

pub type Unknown = OptionalRc<Internal>;

type Result<T> = result::Result<T, ()>;

impl<T: Cast> From<T> for Unknown {
    #[inline(always)]
    fn from(t: T) -> Self {
        t.unknown()
    }
}

impl Unknown {
    #[inline(always)]
    unsafe fn u64(&self) -> u64 {
        self.internal().0
    }
    // generic
    #[inline(always)]
    pub fn is<T: Cast>(&self) -> bool {
        unsafe { T::cast_is(self.u64()) }
    }
    pub fn try_move<T: Cast>(self) -> Result<T> {
        if self.is::<T>() {
            return Ok(unsafe { T::from_unknown_internal(self.move_to_internal().0) });
        }
        Err(())
    }
    //
    #[inline(always)]
    pub fn is_rc(&self) -> bool {
        RC.has(unsafe { self.u64() })
    }
    //
    #[inline(always)]
    pub fn try_ref<T: TagRc>(&self) -> Result<&mut Container<T>> {
        let v = unsafe { self.u64() };
        if T::RC_SUBSET.has(v) {
            let p = (v & PTR_SUBSET_SUPERPOSITION) as *mut Container<T>;
            return Ok(unsafe { &mut *p });
        }
        Err(())
    }
    //
    pub fn get_type(&self) -> Type {
        if self.is_rc() {
            if self.is::<StringRc>() {
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
        object::{ObjectHeader, ObjectRc},
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
        assert_eq!(Unknown::from(1.0).try_move(), Ok(1.0));
        //let y = -1.0;
        let x: Unknown = (-1.0).into();
        assert_eq!(x.try_move(), Ok(-1.0));
        assert_eq!(Unknown::from(f64::INFINITY).try_move(), Ok(f64::INFINITY));
        assert_eq!(
            Unknown::from(f64::NEG_INFINITY).try_move(),
            Ok(f64::NEG_INFINITY)
        );
        assert!(Unknown::from(f64::NAN).try_move::<f64>().unwrap().is_nan());
        //
        assert_eq!(Unknown::from(true).try_move::<f64>(), Err(()));
        assert_eq!(Unknown::from(Null()).try_move::<f64>(), Err(()));
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_bool() {
        assert_eq!(Unknown::from(true).try_move(), Ok(true));
        assert_eq!(Unknown::from(false).try_move(), Ok(false));
        //
        assert_eq!(Unknown::from(15.0).try_move::<bool>(), Err(()));
        assert_eq!(Unknown::from(Null()).try_move::<bool>(), Err(()));
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
        let s = StringRc::alloc(StringHeader(), [].into_iter());
        assert!(Unknown::from(s.clone()).is::<StringRc>());
        let v = s.get_items_mut();
        assert!(v.is_empty());
        //
        assert!(!Unknown::from(15.0).is::<StringRc>());
        assert!(!Unknown::from(true).is::<StringRc>());
        assert!(!Null().unknown().is::<StringRc>());

        let s = StringRc::alloc(StringHeader(), [0x20, 0x21].into_iter());
        assert!(Unknown::from(s.clone()).is::<StringRc>());
        let v = s.get_items_mut();
        assert_eq!(v, [0x20, 0x21]);
        let u = Unknown::from(s);
        {
            let s = u.try_ref::<StringHeader>().unwrap();
            let items = s.get_items_mut();
            assert_eq!(items, [0x20, 0x21]);
        }
        let s = u.try_move::<StringRc>().unwrap();
        let items = s.get_items_mut();
        assert_eq!(items, [0x20, 0x21]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_object() {
        assert!(!Null().unknown().is::<ObjectRc>());

        let o = ObjectRc::alloc(ObjectHeader(), [].into_iter());
        assert!(Unknown::from(o.clone()).is::<ObjectRc>());
        let v = o.get_items_mut();
        assert!(v.is_empty());
        //
        assert!(!15.0.unknown().is::<ObjectRc>());
        assert!(!true.unknown().is::<ObjectRc>());

        let o = ObjectRc::alloc(ObjectHeader(), [].into_iter());
        let u = o.unknown();
        assert_eq!(u.get_type(), Type::Object);
        {
            let o = u.try_move::<ObjectRc>().unwrap();
            let items = o.get_items_mut();
            assert!(items.is_empty());
        }
    }
}

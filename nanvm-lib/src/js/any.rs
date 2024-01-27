use crate::mem::{block::Block, manager::Dealloc, optional_ref::OptionalRef};

use super::{
    any_cast::AnyCast,
    any_internal::AnyInternal,
    bitset::{ref_type, REF_SUBSET_SUPERPOSITION},
    null::Null,
    ref_cast::RefCast,
    type_::Type,
};

// type Result<T> = result::Result<T, ()>;

pub type Any<D> = OptionalRef<AnyInternal<D>>;

impl<D: Dealloc> Any<D> {
    #[inline(always)]
    unsafe fn u64(&self) -> u64 {
        self.internal().0
    }
    #[inline(always)]
    pub fn is<T: AnyCast<D>>(&self) -> bool {
        unsafe { T::has_same_type(self.u64()) }
    }
    /// `T` should have the same allocator as `Any`.
    ///
    /// ```
    /// use nanvm_lib::{js::{any::Any, js_string::JsStringRef}, mem::{manager::Dealloc, ref_::Ref}};
    /// fn dummy<A: Dealloc>(s: JsStringRef<A>) -> Any<A> {
    ///     Any::move_from(s)
    /// }
    /// ```
    ///
    /// ```compile_fail
    /// use nanvm_lib::{js::{any::Any, js_string::JsStringRef}, mem::{manager::Dealloc, ref_::Ref}};
    /// fn dummy<A: Dealloc, B: Dealloc>(s: JsStringRef<A>) -> Any<B> {
    ///     Any::move_from(s)
    /// }
    /// ```
    pub fn move_from<T: AnyCast<D>>(t: T) -> Self {
        t.move_to_any()
    }
    pub fn try_move<T: AnyCast<D>>(self) -> Result<T, ()> {
        if self.is::<T>() {
            return Ok(unsafe { T::from_any_internal(self.move_to_internal().0) });
        }
        Err(())
    }
    //
    pub fn get_type(&self) -> Type {
        if self.is_ref() {
            match ref_type(unsafe { self.internal().0 }) {
                0b00 => Type::String,
                0b01 => Type::Object,
                0b10 => Type::Array,
                _ => unreachable!(),
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
    /// `T` should have the same allocator as `Any`.
    ///
    /// ```
    /// use nanvm_lib::{js::{any::Any, js_string::JsStringRef}, mem::{manager::Dealloc, ref_::Ref}};
    /// fn dummy<A: Dealloc>(a: Any<A>) -> JsStringRef<A> {
    ///     a.try_move().unwrap()
    /// }
    /// ```
    ///
    /// ```compile_fail
    /// use nanvm_lib::{js::{any::Any, js_string::JsStringRef}, mem::{manager::Dealloc, ref_::Ref}};
    /// fn dummy<A: Dealloc, B: Dealloc>(a: Any<A>) -> JsStringRef, B> {
    ///     a.try_move().unwrap()
    /// }
    /// ```
    #[inline(always)]
    pub fn try_ref<T: RefCast<D>>(&self) -> Result<&Block<T, D>, ()> {
        let v = unsafe { self.u64() };
        if T::REF_SUBSET.has(v) {
            let p = (v & REF_SUBSET_SUPERPOSITION) as *const Block<T, D>;
            return Ok(unsafe { &*p });
        }
        Err(())
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        js::{
            js_array::{new_array, JsArrayRef},
            js_object::{new_object, JsObjectRef},
            js_string::{new_string, JsString, JsStringRef},
            null::Null,
        },
        mem::{global::Global},
    };

    use super::*;

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
        type A = Any<Global>;
        assert_eq!(A::move_from(1.0).try_move(), Ok(1.0));
        let x: A = A::move_from(-1.0);
        assert_eq!(x.try_move(), Ok(-1.0));
        assert_eq!(A::move_from(f64::INFINITY).try_move(), Ok(f64::INFINITY));
        assert_eq!(
            A::move_from(f64::NEG_INFINITY).try_move(),
            Ok(f64::NEG_INFINITY)
        );
        assert!(A::move_from(f64::NAN).try_move::<f64>().unwrap().is_nan());
        //
        assert_eq!(A::move_from(true).try_move::<f64>(), Err(()));
        assert_eq!(A::move_from(Null()).try_move::<f64>(), Err(()));
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_bool() {
        type A = Any<Global>;
        assert_eq!(A::move_from(true).try_move(), Ok(true));
        assert_eq!(A::move_from(false).try_move(), Ok(false));
        //
        assert_eq!(A::move_from(15.0).try_move::<bool>(), Err(()));
        assert_eq!(A::move_from(Null()).try_move::<bool>(), Err(()));
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_null() {
        type A = Any<Global>;
        assert!(A::move_from(Null()).is::<Null>());
        //
        assert!(!A::move_from(-15.7).is::<Null>());
        assert!(!A::move_from(false).is::<Null>());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_type() {
        type A = Any<Global>;
        assert_eq!(A::move_from(15.0).get_type(), Type::Number);
        assert_eq!(A::move_from(true).get_type(), Type::Bool);
        assert_eq!(A::move_from(Null()).get_type(), Type::Null);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_string() {
        type A = Any<Global>;
        type StringRef = JsStringRef<Global>;
        let sm = new_string(Global(), [].into_iter());
        let s = sm.to_ref();
        assert!(A::move_from(s.clone()).is::<StringRef>());
        let v = s.items();
        assert!(v.is_empty());

        //
        assert!(!A::move_from(15.0).is::<StringRef>());
        assert!(!A::move_from(true).is::<StringRef>());
        assert!(!A::move_from(Null()).is::<StringRef>());

        let s = new_string(Global(), [0x20, 0x21].into_iter()).to_ref();
        assert!(A::move_from(s.clone()).is::<StringRef>());
        let v = s.items();
        assert_eq!(v, [0x20, 0x21]);
        let u = A::move_from(s);
        {
            let s = u.try_ref::<JsString>().unwrap();
            let items = s.object().items();
            assert_eq!(items, [0x20, 0x21]);
        }
        let s = u.try_move::<StringRef>().unwrap();
        let items = s.items();
        assert_eq!(items, [0x20, 0x21]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_object() {
        type A = Any<Global>;
        type ObjectRef = JsObjectRef<Global>;
        assert!(!A::move_from(Null()).is::<ObjectRef>());

        let o: ObjectRef = new_object(Global(), [].into_iter()).to_ref();
        assert!(A::move_from(o.clone()).is::<ObjectRef>());
        let v = o.items();
        assert!(v.is_empty());
        //
        assert!(!A::move_from(15.0).is::<ObjectRef>());
        assert!(!A::move_from(true).is::<ObjectRef>());

        let o: ObjectRef = new_object(Global(), [].into_iter()).to_ref();
        let u = A::move_from(o);
        assert_eq!(u.get_type(), Type::Object);
        {
            let o = u.try_move::<ObjectRef>().unwrap();
            let items = o.items();
            assert!(items.is_empty());
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_array() {
        type A = Any<Global>;
        type ArrayRef = JsArrayRef<Global>;
        assert!(!A::move_from(Null()).is::<ArrayRef>());

        let o: ArrayRef = new_array(Global(), [].into_iter()).to_ref();
        assert!(A::move_from(o.clone()).is::<ArrayRef>());
        let v = o.items();
        assert!(v.is_empty());
        //
        assert!(!A::move_from(15.0).is::<ArrayRef>());
        assert!(!A::move_from(true).is::<ArrayRef>());

        let o: ArrayRef = new_array(Global(), [].into_iter()).to_ref();
        let u = A::move_from(o);
        assert_eq!(u.get_type(), Type::Array);
        {
            let o = u.try_move::<ArrayRef>().unwrap();
            let items = o.items();
            assert!(items.is_empty());
        }
    }
}

use core::result;

use crate::mem::{block::Block, manager::Dealloc, optional_ref::OptionalRef, ref_::Ref};

use super::{
    any_internal::AnyInternal, bitset::RC_SUBSET_SUPERPOSITION, cast::Cast,
    extension_ref::ExtensionRef, null::Null, string::StringHeader, type_::Type,
};

// type Result<T> = result::Result<T, ()>;

pub type Any<D: Dealloc> = OptionalRef<AnyInternal<D>>;

impl<D: Dealloc> Any<D> {
    #[inline(always)]
    unsafe fn u64(&self) -> u64 {
        self.internal().0
    }
    #[inline(always)]
    pub fn is<T: Cast<D>>(&self) -> bool {
        unsafe { T::is_type_of(self.u64()) }
    }
    pub fn move_from<T: Cast<D>>(t: T) -> Self {
        t.move_to_any()
    }
    pub fn try_move<T: Cast<D>>(self) -> Result<T, ()> {
        if self.is::<T>() {
            return Ok(unsafe { T::from_any_internal(self.move_to_internal().0) });
        }
        Err(())
    }
    //
    pub fn get_type(&self) -> Type {
        if self.is_ref() {
            if self.is::<Ref<StringHeader, D>>() {
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
    //
    #[inline(always)]
    pub fn try_ref<T: ExtensionRef>(&self) -> Result<&Block<T, D>, ()> {
        let v = unsafe { self.u64() };
        if T::REF_SUBSET.has(v) {
            let p = (v & RC_SUBSET_SUPERPOSITION) as *const Block<T, D>;
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
        js::{null::Null, object::ObjectHeader},
        mem::{global::Global, manager::Manager},
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
    fn test_number2() {
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
    fn test_bool2() {
        type A = Any<Global>;
        assert_eq!(A::move_from(true).try_move(), Ok(true));
        assert_eq!(A::move_from(false).try_move(), Ok(false));
        //
        assert_eq!(A::move_from(15.0).try_move::<bool>(), Err(()));
        assert_eq!(A::move_from(Null()).try_move::<bool>(), Err(()));
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_null2() {
        type A = Any<Global>;
        assert!(A::move_from(Null()).is::<Null>());
        //
        assert!(!A::move_from(-15.7).is::<Null>());
        assert!(!A::move_from(false).is::<Null>());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_type2() {
        type A = Any<Global>;
        assert_eq!(A::move_from(15.0).get_type(), Type::Number);
        assert_eq!(A::move_from(true).get_type(), Type::Bool);
        assert_eq!(A::move_from(Null()).get_type(), Type::Null);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_string2() {
        type A = Any<Global>;
        type StringRef = Ref<StringHeader, Global>;
        let sm = Global().flexible_array_new::<u16>([].into_iter());
        let s = sm.to_ref();
        assert!(A::move_from(s.clone()).is::<StringRef>());
        let v = s.items();
        assert!(v.is_empty());

        //
        assert!(!A::move_from(15.0).is::<StringRef>());
        assert!(!A::move_from(true).is::<StringRef>());
        assert!(!A::move_from(Null()).is::<StringRef>());

        let s = Global()
            .flexible_array_new::<u16>([0x20, 0x21].into_iter())
            .to_ref();
        assert!(A::move_from(s.clone()).is::<StringRef>());
        let v = s.items();
        assert_eq!(v, [0x20, 0x21]);
        let u = A::move_from(s);
        {
            let s = u.try_ref::<StringHeader>().unwrap();
            let items = s.object().items();
            assert_eq!(items, [0x20, 0x21]);
        }
        let s = u.try_move::<StringRef>().unwrap();
        let items = s.items();
        assert_eq!(items, [0x20, 0x21]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_object2() {
        type A = Any<Global>;
        type ObjectRc = Ref<ObjectHeader<Global>, Global>;
        assert!(!A::move_from(Null()).is::<ObjectRc>());

        let o = Global().flexible_array_new::<A>([].into_iter()).to_ref();
        assert!(A::move_from(o.clone()).is::<ObjectRc>());
        let v = o.items();
        assert!(v.is_empty());
        //
        assert!(!A::move_from(15.0).is::<ObjectRc>());
        assert!(!A::move_from(true).is::<ObjectRc>());

        let o = Global().flexible_array_new::<A>([].into_iter()).to_ref();
        let u = A::move_from(o);
        assert_eq!(u.get_type(), Type::Object);
        {
            let o = u.try_move::<ObjectRc>().unwrap();
            let items = o.items();
            assert!(items.is_empty());
        }
    }
}

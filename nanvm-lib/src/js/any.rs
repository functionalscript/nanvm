use core::result;

use crate::{
    common::allocator::GlobalAllocator,
    container::{Container, OptionalRc},
};

use super::{
    any_internal::AnyInternal,
    bitset::{RC, RC_SUBSET_SUPERPOSITION},
    cast::Cast,
    extension_rc::ExtensionRc,
    null::Null,
    string::StringRc,
    type_::Type,
};

pub type Any = OptionalRc<AnyInternal>;

type Result<T> = result::Result<T, ()>;

impl<T: Cast> From<T> for Any {
    #[inline(always)]
    fn from(t: T) -> Self {
        t.move_to_any()
    }
}

impl Any {
    #[inline(always)]
    unsafe fn u64(&self) -> u64 {
        self.optional_base().0
    }
    // generic
    #[inline(always)]
    pub fn is<T: Cast>(&self) -> bool {
        unsafe { T::is_type_of(self.u64()) }
    }
    pub fn try_move<T: Cast>(self) -> Result<T> {
        if self.is::<T>() {
            return Ok(unsafe { T::from_any_internal(self.move_to_optional_base().0) });
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
    pub fn try_ref<T: ExtensionRc>(&self) -> Result<&mut Container<T>> {
        let v = unsafe { self.u64() };
        if T::RC_SUBSET.has(v) {
            let p = (v & RC_SUBSET_SUPERPOSITION) as *mut Container<T>;
            return Ok(unsafe { &mut *p });
        }
        Err(())
    }
    //
    pub fn get_type(&self) -> Type {
        if self.is_rc() {
            if self.is::<StringRc<GlobalAllocator>>() {
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

    use crate::{
        common::allocator::GlobalAllocator,
        js::{
            null::Null,
            object::{self, ObjectHeader},
            string::{self, StringHeader},
        },
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
        assert_eq!(Any::from(1.0).try_move(), Ok(1.0));
        //let y = -1.0;
        let x: Any = (-1.0).into();
        assert_eq!(x.try_move(), Ok(-1.0));
        assert_eq!(Any::from(f64::INFINITY).try_move(), Ok(f64::INFINITY));
        assert_eq!(
            Any::from(f64::NEG_INFINITY).try_move(),
            Ok(f64::NEG_INFINITY)
        );
        assert!(Any::from(f64::NAN).try_move::<f64>().unwrap().is_nan());
        //
        assert_eq!(Any::from(true).try_move::<f64>(), Err(()));
        assert_eq!(Any::from(Null()).try_move::<f64>(), Err(()));
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_bool() {
        assert_eq!(true.move_to_any().try_move(), Ok(true));
        assert_eq!(Any::from(false).try_move(), Ok(false));
        //
        assert_eq!(Any::from(15.0).try_move::<bool>(), Err(()));
        assert_eq!(Any::from(Null()).try_move::<bool>(), Err(()));
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_null() {
        assert!(Any::from(Null()).is::<Null>());
        //
        assert!(!Any::from(-15.7).is::<Null>());
        assert!(!Any::from(false).is::<Null>());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_type() {
        assert_eq!(Any::from(15.0).get_type(), Type::Number);
        assert_eq!(Any::from(true).get_type(), Type::Bool);
        assert_eq!(Null().move_to_any().get_type(), Type::Null);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_string() {
        type StringRc = string::StringRc<GlobalAllocator>;
        let s = StringRc::alloc(GlobalAllocator(), StringHeader::default(), [].into_iter());
        assert!(Any::from(s.clone()).is::<StringRc>());
        let v = s.get_items_mut();
        assert!(v.is_empty());
        //
        assert!(!Any::from(15.0).is::<StringRc>());
        assert!(!Any::from(true).is::<StringRc>());
        assert!(!Null().move_to_any().is::<StringRc>());

        let s = StringRc::alloc(
            GlobalAllocator(),
            StringHeader::default(),
            [0x20, 0x21].into_iter(),
        );
        assert!(Any::from(s.clone()).is::<StringRc>());
        let v = s.get_items_mut();
        assert_eq!(v, [0x20, 0x21]);
        let u = Any::from(s);
        {
            let s = u.try_ref::<StringHeader<GlobalAllocator>>().unwrap();
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
        type ObjectRc = object::ObjectRc<GlobalAllocator>;
        assert!(!Null().move_to_any().is::<ObjectRc>());

        let o = ObjectRc::alloc(GlobalAllocator(), ObjectHeader::default(), [].into_iter());
        assert!(Any::from(o.clone()).is::<ObjectRc>());
        let v = o.get_items_mut();
        assert!(v.is_empty());
        //
        assert!(!15.0.move_to_any().is::<ObjectRc>());
        assert!(!true.move_to_any().is::<ObjectRc>());

        let o = ObjectRc::alloc(GlobalAllocator(), ObjectHeader::default(), [].into_iter());
        let u = o.move_to_any();
        assert_eq!(u.get_type(), Type::Object);
        {
            let o = u.try_move::<ObjectRc>().unwrap();
            let items = o.get_items_mut();
            assert!(items.is_empty());
        }
    }
}

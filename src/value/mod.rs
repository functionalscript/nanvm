mod extension;
mod internal;

use crate::{
    container::{Base, Container, Info, Update},
    number,
    object::ObjectHeader,
    ptr_subset::{PtrSubset, PTR_SUBSET_SUPERPOSITION},
    string::StringHeader,
    type_::Type,
};

use self::extension::{BOOL, EXTENSION, FALSE, OBJECT, PTR, STRING, TRUE};

#[derive(Debug)]
#[repr(transparent)]
pub struct Value(u64);

fn update(v: u64, u: Update) -> isize {
    if !PTR.has(v) {
        return 1;
    }
    let i = v & PTR_SUBSET_SUPERPOSITION;
    if i == 0 {
        return 1;
    }
    unsafe { Base::update(&mut *(i as *mut Base), u) }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        let c = self.0;
        update(c, Update::AddRef);
        Self(c)
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        let c = self.0;
        if update(c, Update::Release) != 0 {
            return;
        }
        let p = c & PTR_SUBSET_SUPERPOSITION;
        if STRING.subset().has(c) {
            STRING.dealloc(p as *mut Base);
        } else {
            OBJECT.dealloc(p as *mut Base);
        }
    }
}

impl Value {
    // number
    fn from_number(n: f64) -> Self {
        let n = n.to_bits();
        assert!(number::is_valid(n));
        Self(n)
    }
    const fn is_number(&self) -> bool {
        !EXTENSION.has(self.0)
    }
    fn get_number(&self) -> Option<f64> {
        if self.is_number() {
            return Some(f64::from_bits(self.0));
        }
        None
    }
    // bool
    const fn from_bool(b: bool) -> Self {
        Self(if b { TRUE } else { FALSE })
    }
    const fn is_bool(&self) -> bool {
        BOOL.has(self.0)
    }
    const fn get_bool(&self) -> Option<bool> {
        if self.is_bool() {
            return Some(self.0 != FALSE);
        }
        None
    }
    //
    const fn is_ptr(&self) -> bool {
        PTR.has(self.0)
    }
    //
    const fn null() -> Self {
        Self(OBJECT.subset().tag)
    }
    const fn is_null(&self) -> bool {
        self.0 == OBJECT.subset().tag
    }
    //
    fn create_container<T: Info>(
        ps: &PtrSubset<T>,
        info: T,
        i: impl ExactSizeIterator<Item = T::Item>,
    ) -> Self {
        let p = unsafe { Container::alloc(info, i) } as u64;
        assert!(ps.subset().mask & p == 0);
        Self(p | ps.subset().tag)
    }
    fn get_container<T: Info>(&self, ps: &PtrSubset<T>) -> Option<&mut Container<T>> {
        let v = self.0;
        if ps.subset().has(v) {
            let p = v & PTR_SUBSET_SUPERPOSITION;
            if p == 0 {
                return None;
            }
            return Some(unsafe { &mut *(p as *mut Container<T>) });
        }
        None
    }
    // string
    const fn is_string(&self) -> bool {
        STRING.subset().has(self.0)
    }
    fn from_string(s: impl ExactSizeIterator<Item = u16>) -> Self {
        Self::create_container(&STRING, StringHeader(), s)
    }
    fn get_string(&self) -> Option<&mut Container<StringHeader>> {
        self.get_container(&STRING)
    }
    // object
    const fn is_object(&self) -> bool {
        OBJECT.subset().has(self.0)
    }
    fn from_object(i: impl ExactSizeIterator<Item = (Value, Value)>) -> Self {
        Self::create_container(&OBJECT, ObjectHeader(), i)
    }
    fn get_object(&self) -> Option<&mut Container<ObjectHeader>> {
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

    use super::*;
    use crate::number::NAN;

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
        assert_eq!(Value::from_number(1.0).get_number(), Some(1.0));
        assert_eq!(Value::from_number(-1.0).get_number(), Some(-1.0));
        assert_eq!(
            Value::from_number(f64::INFINITY).get_number(),
            Some(f64::INFINITY)
        );
        assert_eq!(
            Value::from_number(f64::NEG_INFINITY).get_number(),
            Some(f64::NEG_INFINITY)
        );
        assert!(Value::from_number(f64::NAN).get_number().unwrap().is_nan());
        //
        assert_eq!(Value::from_bool(true).get_number(), None);
        assert_eq!(Value::null().get_number(), None);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_bool() {
        assert_eq!(Value::from_bool(true).get_bool(), Some(true));
        assert_eq!(Value::from_bool(false).get_bool(), Some(false));
        //
        assert_eq!(Value::from_number(15.0).get_bool(), None);
        assert_eq!(Value::null().get_bool(), None);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_null() {
        assert!(Value::null().is_null());
        //
        assert!(!Value::from_number(-15.7).is_null());
        assert!(!Value::from_bool(false).is_null());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_object() {
        assert!(Value::null().is_object());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_type() {
        assert_eq!(Value::from_number(15.0).get_type(), Type::Number);
        assert_eq!(Value::from_bool(true).get_type(), Type::Bool);
        assert_eq!(Value::null().get_type(), Type::Object);
    }
}

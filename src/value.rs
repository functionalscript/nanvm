use crate::{
    bit_subset64::BitSubset64,
    container::{Containable, Container, CLONE, DROP},
    number,
    object::Object,
    ptr_subset::{PtrSubset, PTR_SUBSET_SUPERPOSITION},
    string16::String16,
    value_type::ValueType,
};

#[derive(Debug)]
#[repr(transparent)]
pub struct Value(u64);

const EXTENSION: BitSubset64 = BitSubset64::from_tag(0xFFF8_0000_0000_0000);

const EXTENSION_SPLIT: (BitSubset64, BitSubset64) = EXTENSION.split(0x0004_0000_0000_0000);

const BOOL: BitSubset64 = EXTENSION_SPLIT.0;
const PTR: BitSubset64 = EXTENSION_SPLIT.1;

const PTR_SPLIT: (BitSubset64, BitSubset64) = PTR.split(0x0002_0000_0000_0000);

const STRING: PtrSubset<String16> = PTR_SPLIT.0.ptr_subset();
const STRING_TAG: u64 = STRING.subset().tag;
const OBJECT: PtrSubset<Object> = PTR_SPLIT.1.ptr_subset();
const OBJECT_TAG: u64 = OBJECT.subset().tag;

const FALSE: u64 = BOOL.tag;
const TRUE: u64 = BOOL.tag | 1;

fn update<const ADD: bool>(v: u64) {
    if !PTR.has(v) {
        return;
    }
    let p = v & PTR_SUBSET_SUPERPOSITION;
    if p == 0 {
        return;
    }
    if STRING.subset().has(v) {
        STRING.update::<ADD>(p);
    } else {
        OBJECT.update::<ADD>(p);
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        update::<CLONE>(self.0);
        Self(self.0)
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        update::<DROP>(self.0);
    }
}

impl Value {
    fn from_number(n: f64) -> Self {
        let n = n.to_bits();
        assert!(number::is_valid(n));
        Self(n)
    }
    fn from_bool(b: bool) -> Self {
        Self(if b { TRUE } else { FALSE })
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
    const fn is_bool(&self) -> bool {
        BOOL.has(self.0)
    }
    const fn get_bool(&self) -> Option<bool> {
        if self.is_bool() {
            return Some(self.0 != FALSE);
        }
        None
    }
    const fn is_ptr(&self) -> bool {
        PTR.has(self.0)
    }
    const fn is_string(&self) -> bool {
        STRING.subset().has(self.0)
    }
    const fn null() -> Self {
        Self(OBJECT.subset().tag)
    }
    const fn is_null(&self) -> bool {
        self.0 == OBJECT.subset().tag
    }
    const fn is_object(&self) -> bool {
        OBJECT.subset().has(self.0)
    }
    const fn get_type(&self) -> ValueType {
        if self.is_ptr() {
            if self.is_string() {
                ValueType::String
            } else {
                ValueType::Object
            }
        } else {
            if self.is_number() {
                ValueType::Number
            } else {
                ValueType::Bool
            }
        }
    }
    fn get_ptr<T: Containable>(&self, ps: &PtrSubset<T>) -> Option<&mut Container<T>> {
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
    fn get_string(&self) -> Option<&mut Container<String16>> {
        self.get_ptr(&STRING)
    }
    fn get_object(&self) -> Option<&mut Container<Object>> {
        self.get_ptr(&OBJECT)
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use super::*;
    use crate::{const_assert::const_assert, number::NAN};

    const _: () = const_assert(BOOL.has(FALSE));
    const _: () = const_assert(BOOL.has(TRUE));
    const _: () = const_assert(!BOOL.has(0));
    const _: () = const_assert(!BOOL.has(NAN));
    const _: () = const_assert(BOOL.has(EXTENSION.mask));

    #[test]
    fn test_unsized() {
        let _x: Rc<[u8]> = Rc::new([1, 3]);
        // let _y: Rc<(u8, [u8])> = Rc::new((5, [1, 3]));
        // let r = Vec::default();
        // let n = 4 + 4;
        // let _y: Rc<[u8]> = Rc::new([5; n]);
    }

    #[test]
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
    fn test_bool() {
        assert_eq!(Value::from_bool(true).get_bool(), Some(true));
        assert_eq!(Value::from_bool(false).get_bool(), Some(false));
        //
        assert_eq!(Value::from_number(15.0).get_bool(), None);
        assert_eq!(Value::null().get_bool(), None);
    }

    #[test]
    fn test_null() {
        assert!(Value::null().is_null());
        //
        assert!(!Value::from_number(-15.7).is_null());
        assert!(!Value::from_bool(false).is_null());
    }

    #[test]
    fn test_object() {
        assert!(Value::null().is_object());
    }

    #[test]
    fn test_type() {
        assert_eq!(Value::from_number(15.0).get_type(), ValueType::Number);
        assert_eq!(Value::from_bool(true).get_type(), ValueType::Bool);
        assert_eq!(Value::null().get_type(), ValueType::Object);
    }
}

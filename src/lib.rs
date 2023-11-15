use std::{rc::Rc, collections::{BTreeMap, HashMap}};

#[derive(Debug)]
#[repr(transparent)]
struct Value(u64);

// 0x7FF00000_00000000 -> +Inf
// ...                 -> reserved 2^51 - 1
// 0x7FF80000_00000000 -> NaN
// ...                 -> reserved 2^51 - 1
// 0xFFF00000_00000000 -> -Inf
// ...                 -> reserved 2^52 - 1
// total reserved: 2^53 - 3

// 0x7FF0: 00: +Inf, bool
// 0x7FF8: 01: NaN, string
// 0xFFF0: 10: -Inf, array
// 0xFFF8: 11: null, object

// 0x7FF0: +Inf, null, bool. Future: undefined
// 0x7FF1: string
// 0x7FF2: object
// 0x7FF3: array
// 0x7FF4: Future: bigint
// 0x7FF5: Future: Uint8Array
// 0x7FF6: Future: Date
// 0x7FF7: Future:
// 0x7FF8: NaN
// ...
// 0xFFF0: -Inf
// ...
// 0xFFF8:
// ...

// String
// 10x5: 50 bits (base32)
//  9x5: 45 bits (base32)
//  8x6: 48 bits (base64)
//  7x7: 49 bits (base128 ASCII)
//  6x8: 48 bits
//  5x9: 45 bits
// 4x12: 48 bits
// 3x16: 48 bits
// 2x16: 32 bits
// 1x16: 32 bits

const NAN: u64 = 0x7FF80000_00000000;
const FALSE: u64 = 0x7FF00000_00000001;
const TRUE: u64 = 0x7FF00000_00000002;
const PTR: u64 = 0xFFFF0000_00000000;

impl Value {
    fn number(v: f64) -> Self {
        Self(if v.is_nan() { NAN } else { v.to_bits() })
    }
    fn ptr(v: Rc<Dynamic>) -> Self {
        let v = Rc::into_raw(v);
        assert_eq!(v as u64 & PTR, 0);
        Self((v as u64) | PTR)
    }
    const fn null() -> Self {
        Self(PTR)
    }
    const fn bool(v: bool) -> Self {
        Self(if v { TRUE } else { FALSE })
    }
    fn unpack(&self) -> Unpacked {
        match self.0 {
            TRUE => Unpacked::Bool(true),
            FALSE => Unpacked::Bool(false),
            PTR => Unpacked::Null,
            v => {
                if v & PTR == PTR {
                    Unpacked::Ptr(unsafe { &mut *((v & !PTR) as *mut Dynamic) })
                } else {
                    Unpacked::Number(f64::from_bits(v))
                }
            }
        }
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        match self.unpack() {
            Unpacked::Ptr(p) => { unsafe { Rc::from_raw(p) }; },
            _ => (),
        };
    }
}

#[derive(Debug)]
enum Unpacked<'a> {
    Number(f64),
    Null,
    Ptr(&'a mut Dynamic),
    String(&'a [u16]),
    Bool(bool),
}

type String16 = Rc<[u16]>;

#[derive(Debug)]
struct Object {
    /// We can't use HashMap here because we need to preserve the order of indexes.
    integerProperties: BTreeMap<u32, Value>,
    stringProperties: HashMap<String16, Value>,
    /// Order of string properties.
    order: Vec<usize>
}

#[derive(Debug)]
enum Dynamic {
    String(String16),
    Array(Vec<Value>),
    Object(Object),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nan() {
        assert_ne!(f64::NAN, f64::NAN);
        assert_eq!(f64::NAN.to_bits(), NAN);
        //
        assert_eq!(Value::number(f64::NAN).0, NAN);
        if let Unpacked::Number(x) = Value::number(f64::NAN).unpack() {
            assert!(x.is_nan())
        } else {
            panic!()
        }
        if let Unpacked::Number(x) = Value::number(0.0).unpack() {
            assert_eq!(x, 0.0)
        } else {
            panic!()
        }
        //
        let x = Value::null();
        assert_eq!(x.0, PTR);
        if let Unpacked::Null = x.unpack() {} else { panic!() }
        //
        if let Unpacked::Bool(true) = Value::bool(true).unpack() {} else { panic!() }
        if let Unpacked::Bool(false) = Value::bool(false).unpack() {} else { panic!() }
    }
}

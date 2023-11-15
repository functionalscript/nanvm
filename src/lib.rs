#[repr(transparent)]
struct Value(u64);

// 0x7FF00000_00000000 -> +Inf
// ...                 -> reserved 2^51 - 1
// 0x7FF80000_00000000 -> NaN
// ...                 -> reserved 2^51 - 1
// 0xFFF00000_00000000 -> -Inf
// ...                 -> reserved 2^52 - 1
// total reserved: 2^53 - 3

// 0xFFFF : ptr

const NAN: u64 = 0x7FF80000_00000000;
const FALSE: u64 = 0x7FF00000_00000001;
const TRUE: u64 = 0x7FF00000_00000002;
const PTR: u64 = 0xFFFF0000_00000000;

impl Value {
    fn number(v: f64) -> Self {
        Self(if v.is_nan() { NAN } else { v.to_bits() })
    }
    fn ptr(v: *mut Dynamic) -> Self {
        assert_eq!(v as u64 & PTR, 0);
        Self((v as u64) | PTR)
    }
    fn bool(v: bool) -> Self {
        Self(if v { TRUE } else { FALSE })
    }
    fn unpack(&self) -> Unpacked {
        match self.0 {
            TRUE => Unpacked::Bool(true),
            FALSE => Unpacked::Bool(false),
            v => {
                if v & PTR == PTR {
                    Unpacked::Ptr(Ptr((v & !PTR) as *mut Dynamic))
                } else {
                    Unpacked::Number(f64::from_bits(v))
                }
            }
        }
    }
}

#[derive(PartialEq, Debug)]
enum Unpacked {
    Number(f64),
    Ptr(Ptr),
    Bool(bool),
}

type String16 = Vec<u16>;

enum Dynamic {
    String(String16),
    Object(Vec<(String16, Value)>),
    Array(Vec<Value>),
}

#[repr(transparent)]
#[derive(PartialEq, Debug)]
struct Ptr(*mut Dynamic);

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
        assert_eq!(Value::number(0.0).unpack(), Unpacked::Number(0.0));
        //
        let x = Value::ptr(0 as *mut Dynamic);
        assert_eq!(x.0 & PTR, PTR);
        assert_eq!(x.unpack(), Unpacked::Ptr(Ptr(0 as *mut Dynamic)));
        //
        assert_eq!(Value::bool(true).unpack(), Unpacked::Bool(true));
        assert_eq!(Value::bool(false).unpack(), Unpacked::Bool(false));
    }
}

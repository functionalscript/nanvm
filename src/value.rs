use std::marker::PhantomData;

use crate::{
    bit_subset64::BitSubset64,
    container::{Clean, Container},
    object::Object,
    string16::String16,
};

#[derive(Debug)]
#[repr(transparent)]
struct Value(u64);

// compatible with `f64`
const INFINITY: u64 = 0x7FF0_0000_0000_0000;
const NAN: u64 = 0x7FF8_0000_0000_0000;
const NEG_INFINITY: u64 = 0xFFF0_0000_0000_0000;

//

const EXTENSION: BitSubset64 = BitSubset64::from_tag(0xFFF8_0000_0000_0000);

struct PtrSubset<T: Clean>(BitSubset64, PhantomData<T>);

impl<T: Clean> PtrSubset<T> {
    const fn new(s: BitSubset64) -> Self {
        Self(s, PhantomData)
    }
    fn update<const ADD: bool>(&self, v: u64) {
        let v = v & self.0.superposition();
        if v == 0 {
            return;
        }
        unsafe {
            Container::update::<ADD>(v as *mut Container<T>);
        }
    }
}

const EXTENSION_SPLIT: (BitSubset64, BitSubset64) = EXTENSION.split(50);

const BOOL: BitSubset64 = EXTENSION_SPLIT.0;
const PTR: BitSubset64 = EXTENSION_SPLIT.1;

const PTR_SPLIT: (BitSubset64, BitSubset64) = PTR.split(49);

const STRING: PtrSubset<String16> = PtrSubset::new(PTR_SPLIT.0);
const OBJECT: PtrSubset<Object> = PtrSubset::new(PTR_SPLIT.1);

const FALSE: u64 = BOOL.tag;
const TRUE: u64 = BOOL.tag | 1;

fn update<const ADD: bool>(v: u64) {
    if PTR.has(v) {
        if STRING.0.has(v) {
            STRING.update::<ADD>(v);
        } else {
            OBJECT.update::<ADD>(v);
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        update::<true>(self.0);
        Self(self.0)
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        update::<false>(self.0);
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use super::*;
    use crate::const_assert::const_assert;

    const _: () = const_assert(BOOL.has(FALSE));
    const _: () = const_assert(BOOL.has(TRUE));
    const _: () = const_assert(!BOOL.has(0));
    const _: () = const_assert(!BOOL.has(NAN));
    const _: () = const_assert(BOOL.has(EXTENSION.mask));

    #[test]
    fn test_nan() {
        assert_eq!(f64::INFINITY.to_bits(), INFINITY);
        assert_ne!(f64::NAN, f64::NAN);
        assert_eq!(f64::NAN.to_bits(), NAN);
        assert_eq!(f64::NEG_INFINITY.to_bits(), NEG_INFINITY);
    }

    #[test]
    fn test_unsized() {
        let _x: Rc<[u8]> = Rc::new([1, 3]);
        // let _y: Rc<(u8, [u8])> = Rc::new((5, [1, 3]));
        // let r = Vec::default();
        // let n = 4 + 4;
        // let _y: Rc<[u8]> = Rc::new([5; n]);
    }
}
